use std::fs::File;
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use tauri::AppHandle;
use tauri::Emitter;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::core::ProcessManager;
use crate::core::waveform::cache::{CACHE_HEADER_SIZE, CacheState, POINT_SIZE, get_cache_path};
use crate::core::waveform::model::{
    DEFAULT_POINTS_PER_EVENT, StreamWaveformRequest, TOTAL_WAVEFORM_POINTS, WaveformChunkEvent,
    WaveformFinishedEvent,
};
use crate::logger::{log_error, log_info, log_warn};

pub const BYTES_PER_SAMPLE: usize = 4;
pub const BYTES_PER_FRAME: usize = 2 * BYTES_PER_SAMPLE;
const READ_BUFFER_SIZE: usize = 64 * 1024;

const DISPLAY_TARGET_AMP: f64 = 0.9 * 255.0;
const DISPLAY_MAX_GAIN: f64 = 4.0;

fn compute_display_gain(max_peak: u8) -> f32 {
    if max_peak == 0 {
        return 1.0;
    }
    let gain = (DISPLAY_TARGET_AMP / max_peak as f64).min(DISPLAY_MAX_GAIN);
    gain as f32
}

pub fn resolve_points_per_event(points_per_event: Option<usize>) -> usize {
    points_per_event
        .unwrap_or(DEFAULT_POINTS_PER_EVENT)
        .clamp(1, TOTAL_WAVEFORM_POINTS)
}

pub fn event_count_for(points_per_event: usize) -> usize {
    (TOTAL_WAVEFORM_POINTS + points_per_event - 1) / points_per_event
}

fn choose_auto_target_rate(duration_seconds: f64) -> u32 {
    const MIN_RATE: f64 = 2_000.0;
    const MAX_RATE: f64 = 12_000.0;
    const SAMPLES_PER_POINT: f64 = 64.0;
    let candidate = (TOTAL_WAVEFORM_POINTS as f64 * SAMPLES_PER_POINT) / duration_seconds;
    candidate.clamp(MIN_RATE, MAX_RATE).round() as u32
}

pub fn resolve_target_rate(
    duration_seconds: f64,
    target_rate: Option<u32>,
    audio_tracks_sample_rate: Option<u32>,
) -> u32 {
    if let Some(rate) = target_rate {
        return rate.max(1);
    }
    let auto = choose_auto_target_rate(duration_seconds);
    match audio_tracks_sample_rate {
        Some(source_rate) if source_rate > 0 => auto.min(source_rate),
        _ => auto,
    }
}

pub fn quantize_duration_ms(duration: f64) -> f64 {
    (duration * 1000.0).round() / 1000.0
}

pub fn duration_to_expected_frames(duration: f64, sample_rate: u32) -> u64 {
    (duration * f64::from(sample_rate)).round().max(1.0) as u64
}

fn emit_batch(
    app: &AppHandle,
    job_id: &str,
    track_index: u32,
    points_per_event: usize,
    left_rms: Vec<u8>,
    right_rms: Vec<u8>,
    left_peak_up: Vec<u8>,
    left_peak_down: Vec<u8>,
    right_peak_up: Vec<u8>,
    right_peak_down: Vec<u8>,
    chunk_max_peak: u8,
    point_offset: usize,
) -> Result<(), String> {
    let point_count = left_rms.len();
    if point_count == 0 {
        return Ok(());
    }
    let completed_points = point_offset + point_count;
    let chunk_index = point_offset / points_per_event;

    app.emit(
        "waveform://chunk",
        WaveformChunkEvent {
            job_id: job_id.to_owned(),
            track_index,
            chunk_index,
            point_offset,
            point_count,
            total_points: TOTAL_WAVEFORM_POINTS,
            progress: (completed_points as f32 / TOTAL_WAVEFORM_POINTS as f32).min(1.0),
            points_per_event,
            left_rms,
            right_rms,
            left_peak_up,
            left_peak_down,
            right_peak_up,
            right_peak_down,
            chunk_max_peak,
        },
    )
    .map_err(|error| format!("Failed to emit waveform chunk: {error}"))
}

// ── Streaming Worker & State ─────────────────────────────────────────

pub struct WaveformState {
    current_bin_index: usize,
    frame_cursor: u64,

    current_left_sq: f64,
    current_right_sq: f64,

    current_left_min: f32,
    current_left_max: f32,
    current_right_min: f32,
    current_right_max: f32,

    current_count: u64,

    output_left_rms: Vec<u8>,
    output_right_rms: Vec<u8>,

    output_left_up: Vec<u8>,
    output_left_down: Vec<u8>,
    output_right_up: Vec<u8>,
    output_right_down: Vec<u8>,

    emitted_points: usize,
    cache_file: Option<BufWriter<File>>,

    batch_max_peak: u8,
    max_left_peak: u8,
    max_right_peak: u8,
}

#[inline]
fn reduce_interleaved(samples: &[f32]) -> (f32, f32, f32, f32, f32, f32) {
    let mut left_sq = 0.0f32;
    let mut right_sq = 0.0f32;

    let mut left_min = 0.0f32;
    let mut left_max = 0.0f32;

    let mut right_min = 0.0f32;
    let mut right_max = 0.0f32;

    for pair in samples.chunks_exact(2) {
        let l = pair[0];
        let r = pair[1];

        left_sq += l * l;
        right_sq += r * r;

        left_min = left_min.min(l);
        left_max = left_max.max(l);

        right_min = right_min.min(r);
        right_max = right_max.max(r);
    }

    (left_sq, right_sq, left_min, left_max, right_min, right_max)
}

impl WaveformState {
    pub fn new(
        points_per_event: usize,
        initial_emitted: usize,
        cache_file: Option<BufWriter<File>>,
        start_frame: u64,
    ) -> Self {
        Self {
            current_bin_index: initial_emitted,
            frame_cursor: start_frame,

            current_left_sq: 0.0,
            current_right_sq: 0.0,

            current_left_min: 0.0,
            current_left_max: 0.0,
            current_right_min: 0.0,
            current_right_max: 0.0,

            current_count: 0,

            output_left_rms: Vec::with_capacity(points_per_event),
            output_right_rms: Vec::with_capacity(points_per_event),

            output_left_up: Vec::with_capacity(points_per_event),
            output_left_down: Vec::with_capacity(points_per_event),
            output_right_up: Vec::with_capacity(points_per_event),
            output_right_down: Vec::with_capacity(points_per_event),

            emitted_points: initial_emitted,
            cache_file,

            batch_max_peak: 0,
            max_left_peak: 0,
            max_right_peak: 0,
        }
    }

    fn process_run(
        &mut self,
        samples: &[f32],
        expected_frames: u64,
        app: &AppHandle,
        job_id: &str,
        track_index: u32,
        points_per_event: usize,
    ) -> Result<(), String> {
        let total_points = TOTAL_WAVEFORM_POINTS as u64;
        let frame_count = samples.len() / 2;
        let mut offset = 0usize;

        while offset < frame_count {
            if self.current_bin_index >= TOTAL_WAVEFORM_POINTS {
                break;
            }
            let bin_end_frame =
                ((self.current_bin_index as u64 + 1) * expected_frames) / total_points;
            if self.frame_cursor >= bin_end_frame {
                self.finalize_bin(app, job_id, track_index, points_per_event)?;
                continue;
            }
            let run_len = ((bin_end_frame - self.frame_cursor) as usize).min(frame_count - offset);
            let (left_sq, right_sq, left_min, left_max, right_min, right_max) =
                reduce_interleaved(&samples[offset * 2..(offset + run_len) * 2]);

            self.current_left_sq += f64::from(left_sq);
            self.current_right_sq += f64::from(right_sq);

            self.current_left_min = self.current_left_min.min(left_min);
            self.current_left_max = self.current_left_max.max(left_max);

            self.current_right_min = self.current_right_min.min(right_min);
            self.current_right_max = self.current_right_max.max(right_max);

            self.current_count += run_len as u64;
            self.frame_cursor += run_len as u64;
            offset += run_len;
        }
        Ok(())
    }

    fn write_batch_to_cache(&mut self, job_id: &str) {
        let Some(writer) = &mut self.cache_file else {
            return;
        };

        let count = self.output_left_rms.len();
        let mut buffer = Vec::with_capacity(count * POINT_SIZE as usize);

        for i in 0..count {
            buffer.push(self.output_left_rms[i]);
            buffer.push(self.output_right_rms[i]);
            buffer.push(self.output_left_up[i]);
            buffer.push(self.output_left_down[i]);
            buffer.push(self.output_right_up[i]);
            buffer.push(self.output_right_down[i]);
        }

        if let Err(e) = writer.write_all(&buffer).and_then(|_| writer.flush()) {
            log_warn!(
                job_id,
                error = %e,
                "Cache write failed; disabling cache for remainder of job"
            );

            self.cache_file = None;
        }
    }

    fn flush_current_batch(
        &mut self,
        app: &AppHandle,
        job_id: &str,
        track_index: u32,
        points_per_event: usize,
    ) -> Result<(), String> {
        if self.output_left_rms.len() >= points_per_event {
            self.write_batch_to_cache(job_id);
            let offset = self.emitted_points - self.output_left_rms.len();
            let chunk_max_peak = self.batch_max_peak;
            self.batch_max_peak = 0;
            emit_batch(
                app,
                job_id,
                track_index,
                points_per_event,
                std::mem::replace(
                    &mut self.output_left_rms,
                    Vec::with_capacity(points_per_event),
                ),
                std::mem::replace(
                    &mut self.output_right_rms,
                    Vec::with_capacity(points_per_event),
                ),
                std::mem::replace(
                    &mut self.output_left_up,
                    Vec::with_capacity(points_per_event),
                ),
                std::mem::replace(
                    &mut self.output_left_down,
                    Vec::with_capacity(points_per_event),
                ),
                std::mem::replace(
                    &mut self.output_right_up,
                    Vec::with_capacity(points_per_event),
                ),
                std::mem::replace(
                    &mut self.output_right_down,
                    Vec::with_capacity(points_per_event),
                ),
                chunk_max_peak,
                offset,
            )?;
        }
        Ok(())
    }

    fn finalize_bin(
        &mut self,
        app: &AppHandle,
        job_id: &str,
        track_index: u32,
        points_per_event: usize,
    ) -> Result<(), String> {
        if self.current_count > 0 {
            let inv_count = 1.0 / self.current_count as f64;

            let left_rms_norm = (self.current_left_sq * inv_count).sqrt();
            let right_rms_norm = (self.current_right_sq * inv_count).sqrt();

            let left_rms = (left_rms_norm.cbrt() * 255.0).clamp(0.0, 255.0) as u8;
            let right_rms = (right_rms_norm.cbrt() * 255.0).clamp(0.0, 255.0) as u8;

            let left_up = (self.current_left_max.clamp(0.0, 1.0) * 255.0).clamp(0.0, 255.0) as u8;
            let left_down =
                ((-self.current_left_min).clamp(0.0, 1.0) * 255.0).clamp(0.0, 255.0) as u8;

            let right_up = (self.current_right_max.clamp(0.0, 1.0) * 255.0).clamp(0.0, 255.0) as u8;
            let right_down =
                ((-self.current_right_min).clamp(0.0, 1.0) * 255.0).clamp(0.0, 255.0) as u8;

            self.output_left_rms.push(left_rms);
            self.output_right_rms.push(right_rms);

            self.output_left_up.push(left_up);
            self.output_left_down.push(left_down);
            self.output_right_up.push(right_up);
            self.output_right_down.push(right_down);

            // Track batch max and global max
            let point_max = left_up.max(left_down).max(right_up).max(right_down);
            self.batch_max_peak = self.batch_max_peak.max(point_max);
            self.max_left_peak = self.max_left_peak.max(left_up).max(left_down);
            self.max_right_peak = self.max_right_peak.max(right_up).max(right_down);
        } else {
            self.output_left_rms.push(0);
            self.output_right_rms.push(0);

            self.output_left_up.push(0);
            self.output_left_down.push(0);
            self.output_right_up.push(0);
            self.output_right_down.push(0);
        }

        self.emitted_points += 1;
        self.current_bin_index += 1;

        self.current_left_sq = 0.0;
        self.current_right_sq = 0.0;

        self.current_left_min = 0.0;
        self.current_left_max = 0.0;
        self.current_right_min = 0.0;
        self.current_right_max = 0.0;

        self.current_count = 0;

        self.flush_current_batch(app, job_id, track_index, points_per_event)
    }
}

pub async fn run_waveform_job(
    app: &AppHandle,
    job_id: &str,
    request: StreamWaveformRequest,
    ffmpeg_path: &std::path::Path,
    process_manager: &ProcessManager,
    cancel_token: CancellationToken,
) -> Result<(), String> {
    let points_per_event = resolve_points_per_event(request.points_per_event);
    let target_rate = resolve_target_rate(
        request.duration,
        request.target_rate,
        request.audio_tracks_sample_rate,
    );
    let quantized_duration = quantize_duration_ms(request.duration);
    let expected_frames = duration_to_expected_frames(quantized_duration, target_rate);
    let cache_path = get_cache_path(
        &request.video_path,
        request.track_index,
        target_rate,
        points_per_event,
        quantized_duration,
    )?;
    let start_point = request
        .resume_from_point
        .unwrap_or(0)
        .min(TOTAL_WAVEFORM_POINTS);
    let state = CacheState::open(&cache_path, start_point)?;
    let cached_points = state.cached_points;

    let mut cached_max_left_peak: u8 = 0;
    let mut cached_max_right_peak: u8 = 0;

    if cached_points > 0 {
        if let Ok(mut file) = File::open(&cache_path) {
            if file.seek(SeekFrom::Start(CACHE_HEADER_SIZE)).is_ok() {
                let point_size = POINT_SIZE as usize;
                let mut scan = vec![0u8; (cached_points as usize).saturating_mul(point_size)];
                if file.read_exact(&mut scan).is_ok() {
                    let mut ptr = 0usize;
                    while ptr + 5 < scan.len() {
                        // Layout: rms_l, rms_r, up_l, down_l, up_r, down_r
                        let up_l = scan[ptr + 2];
                        let down_l = scan[ptr + 3];
                        let up_r = scan[ptr + 4];
                        let down_r = scan[ptr + 5];

                        cached_max_left_peak = cached_max_left_peak.max(up_l).max(down_l);
                        cached_max_right_peak = cached_max_right_peak.max(up_r).max(down_r);

                        ptr += point_size;
                    }
                }
            }
        }
    }

    // Replay cached data
    if cached_points > start_point {
        if let Ok(mut file) = File::open(&cache_path) {
            let seek_pos = CACHE_HEADER_SIZE + (start_point as u64 * POINT_SIZE);
            if file.seek(SeekFrom::Start(seek_pos)).is_ok() {
                let points_to_emit = cached_points - start_point;
                let mut data = vec![0u8; points_to_emit * POINT_SIZE as usize];
                if file.read_exact(&mut data).is_ok() {
                    let mut offset = start_point;
                    let mut ptr = 0usize;

                    log_info!(
                        job_id,
                        cached_points,
                        start_point,
                        points_to_replay = cached_points - start_point,
                        "Replaying cached waveform data"
                    );

                    while offset < cached_points {
                        let count = (cached_points - offset).min(points_per_event);

                        let mut l_rms = Vec::with_capacity(count);
                        let mut r_rms = Vec::with_capacity(count);
                        let mut l_up = Vec::with_capacity(count);
                        let mut l_down = Vec::with_capacity(count);
                        let mut r_up = Vec::with_capacity(count);
                        let mut r_down = Vec::with_capacity(count);

                        let mut chunk_max: u8 = 0;

                        for _ in 0..count {
                            let up_l = data[ptr + 2];
                            let down_l = data[ptr + 3];
                            let up_r = data[ptr + 4];
                            let down_r = data[ptr + 5];

                            l_rms.push(data[ptr]);
                            r_rms.push(data[ptr + 1]);
                            l_up.push(up_l);
                            l_down.push(down_l);
                            r_up.push(up_r);
                            r_down.push(down_r);

                            chunk_max = chunk_max.max(up_l).max(down_l).max(up_r).max(down_r);
                            ptr += POINT_SIZE as usize;
                        }
                        let _ = emit_batch(
                            app,
                            job_id,
                            request.track_index,
                            points_per_event,
                            l_rms,
                            r_rms,
                            l_up,
                            l_down,
                            r_up,
                            r_down,
                            chunk_max,
                            offset,
                        )
                        .map_err(|e| {
                            log_warn!(job_id, offset, error = %e, "Failed to emit cached waveform chunk")
                        });

                        offset += count;
                    }
                }
            }
        }
    }

    if cached_points >= TOTAL_WAVEFORM_POINTS {
        let max_peak = cached_max_left_peak.max(cached_max_right_peak);
        let display_gain = compute_display_gain(max_peak);

        log_info!(
            job_id,
            cached_points,
            max_left_peak = cached_max_left_peak,
            max_right_peak = cached_max_right_peak,
            max_peak,
            display_gain,
            "Fully cached; skipping FFmpeg decode"
        );

        let _ = app.emit(
            "waveform://finished",
            WaveformFinishedEvent {
                job_id: job_id.to_owned(),
                track_index: request.track_index,
                total_points: TOTAL_WAVEFORM_POINTS,
                decoded_frames: 0,
                expected_frames,
                target_rate,
                max_left_peak: cached_max_left_peak,
                max_right_peak: cached_max_right_peak,
                display_gain,
            },
        );
        return Ok(());
    }

    let start_frame =
        ((cached_points as f64 / TOTAL_WAVEFORM_POINTS as f64) * expected_frames as f64) as u64;
    let mut waveform_state = WaveformState::new(
        points_per_event,
        cached_points,
        state.cache_file,
        start_frame,
    );
    waveform_state.max_left_peak = cached_max_left_peak;
    waveform_state.max_right_peak = cached_max_right_peak;
    let start_time = (cached_points as f64 / TOTAL_WAVEFORM_POINTS as f64) * quantized_duration;

    log_info!(
        job_id,
        video = %request.video_path,
        track = request.track_index,
        target_rate,
        resume_at = cached_points,
        "Starting FFmpeg waveform decode"
    );

    let mut args = vec![
        "-hide_banner",
        "-loglevel",
        "error",
        "-nostdin",
        "-threads",
        "0",
        "-vn",
        "-sn",
        "-dn",
    ];
    let start_time_arg = format!("{:.3}", start_time);
    let track_index_arg = format!("0:{}", request.track_index);
    let target_rate_arg = target_rate.to_string();
    if cached_points > 0 && cached_points < TOTAL_WAVEFORM_POINTS {
        args.extend(["-ss", &start_time_arg]);
    }
    args.extend(&[
        "-i",
        &request.video_path,
        "-map",
        &track_index_arg,
        "-ac",
        "2",
        "-ar",
        &target_rate_arg,
        "-sample_fmt",
        "flt",
        "-c:a",
        "pcm_f32le",
        "-f",
        "f32le",
        "-",
    ]);

    let mut child = Command::new(ffmpeg_path)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start FFmpeg: {e}"))?;

    if let Err(e) = process_manager.attach(&child) {
        let _ = child.kill().await;

        log_error!(
            job_id,
            track = request.track_index,
            target_rate,
            error = %e,
            "Failed to attach waveform FFmpeg to ProcessManager"
        );

        return Err(format!("Failed to attach to ProcessManager: {e}"));
    }

    let stdout = child.stdout.take().ok_or("Failed to open FFmpeg stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to open FFmpeg stderr")?;

    let stderr_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stderr);
        let mut bytes = Vec::new();
        let _ = reader.read_to_end(&mut bytes).await;
        String::from_utf8_lossy(&bytes).trim().to_owned()
    });

    let mut reader = BufReader::with_capacity(READ_BUFFER_SIZE, stdout);
    let mut read_buffer = vec![0_u8; READ_BUFFER_SIZE];
    let mut pending = Vec::<u8>::with_capacity(READ_BUFFER_SIZE + BYTES_PER_FRAME);
    let mut sample_buffer: Vec<f32> =
        Vec::with_capacity((READ_BUFFER_SIZE + BYTES_PER_FRAME) / BYTES_PER_SAMPLE);
    let mut decoded_frames = 0_u64;

    loop {
        tokio::select! {
            biased;
            _ = cancel_token.cancelled() => {
                let _ = child.kill().await;

                log_info!(job_id, "Waveform job cancelled by user");

                return Err("Job cancelled by user".into());
            }
            read_result = reader.read(&mut read_buffer) => {
                let bytes_read = match read_result {
                    Ok(n) => n,
                    Err(error) => return Err(format!("Failed reading FFmpeg stdout: {error}")),
                };
                if bytes_read == 0 {
                    break;
                }
                pending.extend_from_slice(&read_buffer[..bytes_read]);
                let complete_bytes = pending.len() - (pending.len() % BYTES_PER_FRAME);
                if complete_bytes > 0 {
                    sample_buffer.clear();
                    sample_buffer.extend_from_slice(
                        bytemuck::cast_slice::<u8, f32>(&pending[..complete_bytes]),
                    );
                    waveform_state.process_run(
                        &sample_buffer,
                        expected_frames,
                        app,
                        job_id,
                        request.track_index,
                        points_per_event,
                    )?;
                    decoded_frames += (complete_bytes / BYTES_PER_FRAME) as u64;
                    let remainder = pending.len() - complete_bytes;
                    pending.copy_within(complete_bytes.., 0);
                    pending.truncate(remainder);
                }
            }
        }
    }

    if waveform_state.current_count > 0 {
        waveform_state.finalize_bin(app, job_id, request.track_index, points_per_event)?;
    }

    if !waveform_state.output_left_rms.is_empty() {
        waveform_state.write_batch_to_cache(job_id);
        let offset = waveform_state.emitted_points - waveform_state.output_left_rms.len();
        let chunk_max_peak = waveform_state.batch_max_peak;
        waveform_state.batch_max_peak = 0;
        emit_batch(
            app,
            job_id,
            request.track_index,
            waveform_state.output_left_rms.len(),
            std::mem::replace(
                &mut waveform_state.output_left_rms,
                Vec::with_capacity(points_per_event),
            ),
            std::mem::replace(
                &mut waveform_state.output_right_rms,
                Vec::with_capacity(points_per_event),
            ),
            std::mem::replace(
                &mut waveform_state.output_left_up,
                Vec::with_capacity(points_per_event),
            ),
            std::mem::replace(
                &mut waveform_state.output_left_down,
                Vec::with_capacity(points_per_event),
            ),
            std::mem::replace(
                &mut waveform_state.output_right_up,
                Vec::with_capacity(points_per_event),
            ),
            std::mem::replace(
                &mut waveform_state.output_right_down,
                Vec::with_capacity(points_per_event),
            ),
            chunk_max_peak,
            offset,
        )?;
    }

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed waiting for FFmpeg: {e}"))?;
    let stderr_message = stderr_task
        .await
        .unwrap_or_else(|e| format!("Failed joining stderr task: {e}"));

    if !status.success() {
        return Err(if stderr_message.is_empty() {
            format!("FFmpeg exited with status {status}")
        } else {
            format!("FFmpeg exited with status {status}: {stderr_message}")
        });
    }

    while waveform_state.current_bin_index < TOTAL_WAVEFORM_POINTS {
        waveform_state.finalize_bin(app, job_id, request.track_index, points_per_event)?;
    }

    if !waveform_state.output_left_rms.is_empty() {
        waveform_state.write_batch_to_cache(job_id);
        let offset = waveform_state.emitted_points - waveform_state.output_left_rms.len();

        let chunk_max_peak = waveform_state.batch_max_peak;

        emit_batch(
            app,
            job_id,
            request.track_index,
            waveform_state.output_left_rms.len(),
            std::mem::replace(
                &mut waveform_state.output_left_rms,
                Vec::with_capacity(points_per_event),
            ),
            std::mem::replace(
                &mut waveform_state.output_right_rms,
                Vec::with_capacity(points_per_event),
            ),
            std::mem::replace(
                &mut waveform_state.output_left_up,
                Vec::with_capacity(points_per_event),
            ),
            std::mem::replace(
                &mut waveform_state.output_left_down,
                Vec::with_capacity(points_per_event),
            ),
            std::mem::replace(
                &mut waveform_state.output_right_up,
                Vec::with_capacity(points_per_event),
            ),
            std::mem::replace(
                &mut waveform_state.output_right_down,
                Vec::with_capacity(points_per_event),
            ),
            chunk_max_peak,
            offset,
        )?;
    }

    let max_left_peak = waveform_state.max_left_peak;
    let max_right_peak = waveform_state.max_right_peak;
    let max_peak = max_left_peak.max(max_right_peak);
    let display_gain = compute_display_gain(max_peak);

    log_info!(
        job_id,
        emitted_points = waveform_state.emitted_points,
        decoded_frames,
        max_left_peak,
        max_right_peak,
        max_peak,
        display_gain,
        "Waveform generation completed"
    );

    let _ = app.emit(
        "waveform://finished",
        WaveformFinishedEvent {
            job_id: job_id.to_owned(),
            track_index: request.track_index,
            total_points: TOTAL_WAVEFORM_POINTS,
            decoded_frames,
            expected_frames,
            target_rate,
            max_left_peak,
            max_right_peak,
            display_gain,
        },
    );

    Ok(())
}
