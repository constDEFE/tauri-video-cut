use std::time::Duration;
use tauri::AppHandle;
use tauri::Emitter;
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::core::ProcessManager;
use crate::core::waveform::cache::{
    CacheWriter, POINT_SIZE, decode_point, get_cache_path, probe, read_raw_points,
};
use crate::core::waveform::model::{
    DEFAULT_POINTS_PER_EVENT, StreamWaveformRequest, TOTAL_WAVEFORM_POINTS, WaveformChunkEvent,
    WaveformFinishedEvent,
};
use crate::logger::{log_error, log_info, log_warn};
use crate::utils::fsx::CacheLock;

pub const BYTES_PER_SAMPLE: usize = 2;
pub const BYTES_PER_FRAME: usize = 2 * BYTES_PER_SAMPLE;
const READ_BUFFER_SIZE: usize = 1024 * 1024;

const DISPLAY_TARGET_AMP: f64 = 0.9 * 255.0;
const DISPLAY_MAX_GAIN: f64 = 4.0;

const S16_TO_F32: f32 = 1.0 / 32768.0;

pub fn compute_display_gain(max_peak: u8) -> f32 {
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

    let candidate =
        (TOTAL_WAVEFORM_POINTS as f64 * SAMPLES_PER_POINT) / duration_seconds.max(0.001);
    candidate.clamp(MIN_RATE, MAX_RATE).round() as u32
}

pub fn resolve_target_rate(
    duration_seconds: f64,
    target_rate: Option<u32>,
    audio_tracks_sample_rate: Option<u32>,
) -> u32 {
    const MAX_ALLOWED_RATE: u32 = 12_000;
    if let Some(rate) = target_rate {
        let mut resolved = rate.clamp(1, MAX_ALLOWED_RATE);
        if let Some(source_rate) = audio_tracks_sample_rate.filter(|&s| s > 0) {
            resolved = resolved.min(source_rate);
        }
        return resolved;
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
    display_gain: f32,
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
            display_gain,
        },
    )
    .map_err(|error| format!("Failed to emit waveform chunk: {error}"))
}

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
    cache_writer: Option<CacheWriter>,
    batch_max_peak: u8,
    max_left_peak: u8,
    max_right_peak: u8,
}

#[inline]
fn reduce_interleaved_s16(raw: &[u8]) -> (f64, f64, f32, f32, f32, f32) {
    let mut left_sq = 0.0f64;
    let mut right_sq = 0.0f64;

    let mut left_min = 0.0f32;
    let mut left_max = 0.0f32;

    let mut right_min = 0.0f32;
    let mut right_max = 0.0f32;

    for frame in raw.chunks_exact(BYTES_PER_FRAME) {
        let l_i16 = i16::from_le_bytes([frame[0], frame[1]]);
        let r_i16 = i16::from_le_bytes([frame[2], frame[3]]);

        let l = l_i16 as f32 * S16_TO_F32;
        let r = r_i16 as f32 * S16_TO_F32;

        let l64 = f64::from(l);
        let r64 = f64::from(r);

        left_sq += l64 * l64;
        right_sq += r64 * r64;

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
        cache_writer: Option<CacheWriter>,
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
            cache_writer,
            batch_max_peak: 0,
            max_left_peak: 0,
            max_right_peak: 0,
        }
    }

    pub fn publish_cache(&mut self) {
        if let Some(w) = self.cache_writer.as_mut() {
            w.publish();
        }
    }

    fn process_run(
        &mut self,
        raw: &[u8],
        expected_frames: u64,
        app: &AppHandle,
        job_id: &str,
        track_index: u32,
        points_per_event: usize,
    ) -> Result<(), String> {
        let total_points = TOTAL_WAVEFORM_POINTS as u64;
        let frame_count = raw.len() / BYTES_PER_FRAME;

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

            let start_byte = offset * BYTES_PER_FRAME;
            let end_byte = (offset + run_len) * BYTES_PER_FRAME;

            let (left_sq, right_sq, left_min, left_max, right_min, right_max) =
                reduce_interleaved_s16(&raw[start_byte..end_byte]);

            self.current_left_sq += left_sq;
            self.current_right_sq += right_sq;

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
        let Some(writer) = &mut self.cache_writer else {
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
        if let Err(e) = writer.write_batch(&buffer) {
            log_warn!(
                job_id,
                error = %e,
                "Cache write failed; disabling cache for remainder of job"
            );

            self.cache_writer = None;
        }
    }

    fn drain_outputs(
        &mut self,
        app: &AppHandle,
        job_id: &str,
        track_index: u32,
        points_per_event: usize,
    ) -> Result<(), String> {
        if self.output_left_rms.is_empty() {
            return Ok(());
        }
        self.write_batch_to_cache(job_id);
        let offset = self.emitted_points - self.output_left_rms.len();
        let chunk_max_peak = self.batch_max_peak;
        self.batch_max_peak = 0;
        let display_gain = compute_display_gain(self.max_left_peak.max(self.max_right_peak));
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
            display_gain,
        )
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

        if self.output_left_rms.len() >= points_per_event {
            self.drain_outputs(app, job_id, track_index, points_per_event)?;
        }

        Ok(())
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
        quantized_duration,
    )?;

    let lock_path = cache_path.with_extension("lock");
    let _lock = match CacheLock::try_acquire(&lock_path) {
        Some(l) => Some(l),
        None => {
            let mut l = None;
            for _ in 0..50 {
                tokio::time::sleep(Duration::from_millis(100)).await;
                if let Some(got) = CacheLock::try_acquire(&lock_path) {
                    l = Some(got);
                    break;
                }
            }
            l
        }
    };

    let start_point = request
        .resume_from_point
        .unwrap_or(0)
        .min(TOTAL_WAVEFORM_POINTS);

    let cached_points = probe(&cache_path);

    let prefix: Vec<u8> = if cached_points > 0 {
        read_raw_points(&cache_path, 0, cached_points).unwrap_or_default()
    } else {
        Vec::new()
    };
    let resume_points = prefix.len() / POINT_SIZE as usize;

    let mut cached_max_left_peak: u8 = 0;
    let mut cached_max_right_peak: u8 = 0;
    for chunk in prefix.chunks_exact(POINT_SIZE as usize) {
        let (_rl, _rr, up_l, down_l, up_r, down_r) = decode_point(chunk);
        cached_max_left_peak = cached_max_left_peak.max(up_l).max(down_l);
        cached_max_right_peak = cached_max_right_peak.max(up_r).max(down_r);
    }

    if resume_points > start_point {
        let replay_start_idx = start_point * POINT_SIZE as usize;
        let replay_data = &prefix[replay_start_idx..];

        log_info!(
            job_id,
            cached_points,
            start_point,
            points_to_replay = resume_points - start_point,
            "Replaying cached waveform data"
        );
        let mut offset = start_point;
        let mut ptr = 0usize;
        while offset < resume_points {
            let count = (resume_points - offset).min(points_per_event);
            let mut l_rms = Vec::with_capacity(count);
            let mut r_rms = Vec::with_capacity(count);
            let mut l_up = Vec::with_capacity(count);
            let mut l_down = Vec::with_capacity(count);
            let mut r_up = Vec::with_capacity(count);
            let mut r_down = Vec::with_capacity(count);
            let mut chunk_max: u8 = 0;
            for _ in 0..count {
                if ptr + POINT_SIZE as usize > replay_data.len() {
                    break;
                }
                let (rl, rr, up_l, down_l, up_r, down_r) =
                    decode_point(&replay_data[ptr..ptr + POINT_SIZE as usize]);
                l_rms.push(rl);
                r_rms.push(rr);
                l_up.push(up_l);
                l_down.push(down_l);
                r_up.push(up_r);
                r_down.push(down_r);
                chunk_max = chunk_max.max(up_l).max(down_l).max(up_r).max(down_r);
                ptr += POINT_SIZE as usize;
            }
            let chunk_gain = compute_display_gain(cached_max_left_peak.max(cached_max_right_peak));
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
                chunk_gain,
            )
            .map_err(
                |e| log_warn!(job_id, offset, error = %e, "Failed to emit cached waveform chunk"),
            );
            offset += count;
        }
    }

    if resume_points >= TOTAL_WAVEFORM_POINTS {
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

    log_info!(
        job_id,
        video = %request.video_path,
        track = request.track_index,
        target_rate,
        resume_at = resume_points,
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

    let start_frame =
        ((resume_points as f64 / TOTAL_WAVEFORM_POINTS as f64) * expected_frames as f64) as u64;

    let cache_writer = CacheWriter::create(&cache_path, job_id, &prefix);

    let mut waveform_state =
        WaveformState::new(points_per_event, resume_points, cache_writer, start_frame);
    waveform_state.max_left_peak = cached_max_left_peak;
    waveform_state.max_right_peak = cached_max_right_peak;

    let start_time = start_frame as f64 / target_rate as f64;

    let start_time_arg = format!("{:.6}", start_time);
    let track_index_arg = format!("0:{}", request.track_index);
    let target_rate_arg = target_rate.to_string();

    if resume_points > 0 && resume_points < TOTAL_WAVEFORM_POINTS {
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
        "s16",
        "-c:a",
        "pcm_s16le",
        "-f",
        "s16le",
        "-",
    ]);

    let mut child = Command::new(ffmpeg_path)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
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
    let mut decoded_frames = 0_u64;
    let mut completed = false;

    loop {
        tokio::select! {
            biased;
            _ = cancel_token.cancelled() => {
                let _ = child.kill().await;
                let _ = child.wait().await;

                log_info!(job_id, "Waveform job cancelled by user");

                return Err("Job cancelled by user".into());
            }
            read_result = reader.read(&mut read_buffer) => {
                let bytes_read = match read_result {
                    Ok(n) => n,
                    Err(error) => {
                        let _ = child.kill().await;
                        let _ = child.wait().await;
                        return Err(format!("Failed reading FFmpeg stdout: {error}"));
                    }
                };
                if bytes_read == 0 { break; }

                pending.extend_from_slice(&read_buffer[..bytes_read]);
                let complete_bytes = pending.len() - (pending.len() % BYTES_PER_FRAME);
                if complete_bytes > 0 {
                    waveform_state.process_run(
                        &pending[..complete_bytes],
                        expected_frames,
                        app,
                        job_id,
                        request.track_index,
                        points_per_event,
                    )?;
                    decoded_frames += (complete_bytes / BYTES_PER_FRAME) as u64;
                    if waveform_state.current_bin_index >= TOTAL_WAVEFORM_POINTS {
                        completed = true;
                        let _ = child.kill().await;
                        break;
                    }
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
    waveform_state.drain_outputs(app, job_id, request.track_index, points_per_event)?;

    let status = child
        .wait()
        .await
        .map_err(|e| format!("Failed waiting for FFmpeg: {e}"))?;

    if !completed && !status.success() {
        let stderr_message = stderr_task.await.unwrap_or_default();

        return Err(if stderr_message.is_empty() {
            format!("FFmpeg exited with status {status}")
        } else {
            format!("FFmpeg exited with status {status}: {stderr_message}")
        });
    }

    if !completed {
        while waveform_state.current_bin_index < TOTAL_WAVEFORM_POINTS {
            waveform_state.finalize_bin(app, job_id, request.track_index, points_per_event)?;
        }
        waveform_state.drain_outputs(app, job_id, request.track_index, points_per_event)?;
    }

    waveform_state.publish_cache();

    if resume_points > 0 {
        let expected_remaining = expected_frames.saturating_sub(start_frame);
        let drift = decoded_frames.abs_diff(expected_remaining);
        if drift > expected_remaining.max(1) / 50 {
            log_warn!(
                job_id,
                drift,
                expected_remaining,
                decoded_frames,
                "Resume decode length deviates >2% from expectation"
            );
        }
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
