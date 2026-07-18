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

// ── Helpers ──────────────────────────────────────────────────────────

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
    left_peak: Vec<u8>,
    right_peak: Vec<u8>,
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
            left_peak,
            right_peak,
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
    current_peak_left_abs: f32,
    current_peak_right_abs: f32,
    current_count: u64,

    output_left_rms: Vec<u8>,
    output_right_rms: Vec<u8>,
    output_left_peak: Vec<u8>,
    output_right_peak: Vec<u8>,

    emitted_points: usize,
    cache_file: Option<BufWriter<File>>,
}

#[inline]
fn reduce_interleaved(samples: &[f32]) -> (f32, f32, f32, f32) {
    let mut left_sq = 0.0f32;
    let mut right_sq = 0.0f32;
    let mut left_peak = 0.0f32;
    let mut right_peak = 0.0f32;
    for pair in samples.chunks_exact(2) {
        let l = pair[0];
        let r = pair[1];
        left_sq += l * l;
        right_sq += r * r;
        left_peak = left_peak.max(l.abs());
        right_peak = right_peak.max(r.abs());
    }
    (left_sq, right_sq, left_peak, right_peak)
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
            current_peak_left_abs: 0.0,
            current_peak_right_abs: 0.0,
            current_count: 0,
            output_left_rms: Vec::with_capacity(points_per_event),
            output_right_rms: Vec::with_capacity(points_per_event),
            output_left_peak: Vec::with_capacity(points_per_event),
            output_right_peak: Vec::with_capacity(points_per_event),
            emitted_points: initial_emitted,
            cache_file,
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
            let (left_sq, right_sq, left_peak, right_peak) =
                reduce_interleaved(&samples[offset * 2..(offset + run_len) * 2]);

            self.current_left_sq += f64::from(left_sq);
            self.current_right_sq += f64::from(right_sq);
            self.current_peak_left_abs = self.current_peak_left_abs.max(left_peak);
            self.current_peak_right_abs = self.current_peak_right_abs.max(right_peak);
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
        let mut ok = true;
        for i in 0..count {
            ok &= writer
                .write_all(&self.output_left_rms[i].to_le_bytes())
                .is_ok();
            ok &= writer
                .write_all(&self.output_right_rms[i].to_le_bytes())
                .is_ok();
            ok &= writer
                .write_all(&self.output_left_peak[i].to_le_bytes())
                .is_ok();
            ok &= writer
                .write_all(&self.output_right_peak[i].to_le_bytes())
                .is_ok();
        }
        ok &= writer.flush().is_ok();

        if !ok {
            log_warn!(
                job_id,
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
            emit_batch(
                app,
                job_id,
                track_index,
                points_per_event,
                std::mem::take(&mut self.output_left_rms),
                std::mem::take(&mut self.output_right_rms),
                std::mem::take(&mut self.output_left_peak),
                std::mem::take(&mut self.output_right_peak),
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

            let left_peak = (self.current_peak_left_abs as f64 * 255.0).clamp(0.0, 255.0) as u8;
            let right_peak = (self.current_peak_right_abs as f64 * 255.0).clamp(0.0, 255.0) as u8;

            self.output_left_rms.push(left_rms);
            self.output_right_rms.push(right_rms);
            self.output_left_peak.push(left_peak);
            self.output_right_peak.push(right_peak);
        } else {
            self.output_left_rms.push(0);
            self.output_right_rms.push(0);
            self.output_left_peak.push(0);
            self.output_right_peak.push(0);
        }

        self.emitted_points += 1;
        self.current_bin_index += 1;

        self.current_left_sq = 0.0;
        self.current_right_sq = 0.0;
        self.current_peak_left_abs = 0.0;
        self.current_peak_right_abs = 0.0;
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
                        let mut l_peak = Vec::with_capacity(count);
                        let mut r_peak = Vec::with_capacity(count);
                        for _ in 0..count {
                            l_rms.push(data[ptr]);
                            r_rms.push(data[ptr + 1]);
                            l_peak.push(data[ptr + 2]);
                            r_peak.push(data[ptr + 3]);
                            ptr += 4;
                        }
                        let _ = emit_batch(
                            app,
                            job_id,
                            request.track_index,
                            points_per_event,
                            l_rms,
                            r_rms,
                            l_peak,
                            r_peak,
                            offset,
                        )
                        .map_err(|e| log_warn!(job_id, offset, error = %e, "Failed to emit cached waveform chunk"));
                        offset += count;
                    }
                }
            }
        }
    }

    if cached_points >= TOTAL_WAVEFORM_POINTS {
        log_info!(
            job_id,
            cached_points,
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

    let start_time = (cached_points as f64 / TOTAL_WAVEFORM_POINTS as f64) * quantized_duration;

    log_info!(job_id, video = %request.video_path, track = request.track_index, target_rate, resume_at = cached_points, "Starting FFmpeg waveform decode");

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

        log_error!(job_id, track = request.track_index, target_rate, error = %e, "Failed to attach waveform FFmpeg to ProcessManager");

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
    // Pre-allocated aligned buffer for bytemuck transmute
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
                    // Transmute bytes → f32 pairs via bytemuck (zero-copy on LE)
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
        emit_batch(
            app,
            job_id,
            request.track_index,
            waveform_state.output_left_rms.len(),
            std::mem::take(&mut waveform_state.output_left_rms),
            std::mem::take(&mut waveform_state.output_right_rms),
            std::mem::take(&mut waveform_state.output_left_peak),
            std::mem::take(&mut waveform_state.output_right_peak),
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
        emit_batch(
            app,
            job_id,
            request.track_index,
            waveform_state.output_left_rms.len(),
            std::mem::take(&mut waveform_state.output_left_rms),
            std::mem::take(&mut waveform_state.output_right_rms),
            std::mem::take(&mut waveform_state.output_left_peak),
            std::mem::take(&mut waveform_state.output_right_peak),
            offset,
        )?;
    }

    log_info!(
        job_id,
        emitted_points = waveform_state.emitted_points,
        decoded_frames,
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
        },
    );

    Ok(())
}
