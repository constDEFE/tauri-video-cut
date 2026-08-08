use crate::core::waveform::cache::{POINT_SIZE, decode_point, probe, read_raw_points};
use crate::core::waveform::engine::{
    compute_display_gain, duration_to_expected_frames, event_count_for, quantize_duration_ms,
    resolve_points_per_event, resolve_target_rate, run_waveform_job,
};
use crate::core::waveform::model::{
    StartWaveformResponse, StreamWaveformRequest, TOTAL_WAVEFORM_POINTS, WaveformChunkEvent,
    WaveformErrorEvent,
};
use crate::core::waveform::registry::WaveformJobRegistry;
use crate::core::waveform::registry::next_job_id;
use crate::logger::{log_debug, log_error, log_info};
use crate::utils::paths::get_ffmpeg_path;
use tauri::{AppHandle, Emitter, State};
use tokio_util::sync::CancellationToken;

#[tauri::command]
pub async fn stream_waveform(
    app: AppHandle,
    request: StreamWaveformRequest,
    process_manager: State<'_, crate::core::ProcessManager>,
    registry: State<'_, WaveformJobRegistry>,
) -> Result<StartWaveformResponse, String> {
    if !std::path::Path::new(&request.video_path).is_file() {
        return Err(format!("Media file does not exist: {}", request.video_path));
    }
    if !request.duration.is_finite() || request.duration <= 0.0 {
        return Err("duration must be a finite number greater than zero".into());
    }

    let job_id = next_job_id();

    let points_per_event = resolve_points_per_event(request.points_per_event);
    let target_rate = resolve_target_rate(
        request.duration,
        request.target_rate,
        request.audio_tracks_sample_rate,
    );

    let quantized_duration = quantize_duration_ms(request.duration);
    let _expected_frames = duration_to_expected_frames(quantized_duration, target_rate);

    let cache_path = crate::core::waveform::cache::get_cache_path(
        &request.video_path,
        request.track_index,
        target_rate,
        quantized_duration,
    )?;

    let cache_key = cache_path.to_string_lossy().into_owned();

    if let Some(old_job_id) = registry.evict(&cache_key) {
        let reg_clone = registry.inner().clone();
        let old_id = old_job_id.clone();
        let _ = tokio::time::timeout(std::time::Duration::from_millis(500), async move {
            while reg_clone.is_running(&old_id) {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await;
    }

    let cached_points = probe(&cache_path);
    if cached_points >= TOTAL_WAVEFORM_POINTS {
        log_info!(
            job_id = %job_id,
            cached_points,
            track = request.track_index,
            "Waveform fully cached; returning from cache"
        );

        if let Ok(data) = read_raw_points(&cache_path, 0, cached_points) {
            let mut left_rms = Vec::with_capacity(cached_points);
            let mut right_rms = Vec::with_capacity(cached_points);
            let mut left_peak_up = Vec::with_capacity(cached_points);
            let mut left_peak_down = Vec::with_capacity(cached_points);
            let mut right_peak_up = Vec::with_capacity(cached_points);
            let mut right_peak_down = Vec::with_capacity(cached_points);
            let mut chunk_max_peak: u8 = 0;
            for chunk in data.chunks_exact(POINT_SIZE as usize) {
                let (rl, rr, up_l, down_l, up_r, down_r) = decode_point(chunk);
                left_rms.push(rl);
                right_rms.push(rr);
                left_peak_up.push(up_l);
                left_peak_down.push(down_l);
                right_peak_up.push(up_r);
                right_peak_down.push(down_r);
                chunk_max_peak = chunk_max_peak.max(up_l).max(down_l).max(up_r).max(down_r);
            }
            let display_gain = compute_display_gain(chunk_max_peak);
            return Ok(StartWaveformResponse {
                job_id: job_id.clone(),
                total_points: TOTAL_WAVEFORM_POINTS,
                points_per_event,
                event_count: event_count_for(points_per_event),
                target_rate,
                cached_data: Some(WaveformChunkEvent {
                    job_id: job_id.clone(),
                    track_index: request.track_index,
                    chunk_index: 0,
                    point_offset: 0,
                    point_count: cached_points,
                    total_points: TOTAL_WAVEFORM_POINTS,
                    progress: 1.0,
                    points_per_event,
                    left_rms,
                    right_rms,
                    left_peak_up,
                    left_peak_down,
                    right_peak_up,
                    right_peak_down,
                    chunk_max_peak,
                    display_gain,
                }),
            });
        }
    }

    let cancel_token = CancellationToken::new();
    registry.register(job_id.clone(), cancel_token.clone(), cache_key);

    let task_job_id = job_id.clone();
    let task_ffmpeg_path = get_ffmpeg_path(&app).map_err(|e| e.to_string())?;
    let task_pm = process_manager.inner().clone();
    let task_registry = registry.inner().clone();
    let task_track_index = request.track_index;
    let task_duration = request.duration;
    let task_target_rate = request.target_rate;
    let task_sample_rate = request.audio_tracks_sample_rate;
    let task_points_per_event = request.points_per_event;

    tauri::async_runtime::spawn(async move {
        let result = run_waveform_job(
            &app,
            &task_job_id,
            request,
            &task_ffmpeg_path,
            &task_pm,
            cancel_token,
        )
        .await;

        if let Err(message) = result {
            if message != "Job cancelled by user" {
                log_error!(
                    job_id = %task_job_id,
                    track = task_track_index,
                    error = %message,
                    "Waveform streaming failed"
                );

                let _ = app.emit(
                    "waveform://error",
                    WaveformErrorEvent {
                        job_id: task_job_id.clone(),
                        track_index: task_track_index,
                        message,
                    },
                );
            } else {
                log_debug!(job_id = %task_job_id, "Waveform job cancel requested");
            }
        }

        task_registry.remove(&task_job_id);
    });

    let points_per_event = resolve_points_per_event(task_points_per_event);
    let target_rate = resolve_target_rate(task_duration, task_target_rate, task_sample_rate);

    Ok(StartWaveformResponse {
        job_id,
        total_points: TOTAL_WAVEFORM_POINTS,
        points_per_event,
        event_count: event_count_for(points_per_event),
        target_rate,
        cached_data: None,
    })
}

#[tauri::command]
pub async fn cancel_waveform(
    job_id: String,
    registry: State<'_, WaveformJobRegistry>,
) -> Result<(), String> {
    registry.cancel(&job_id);
    Ok(())
}
