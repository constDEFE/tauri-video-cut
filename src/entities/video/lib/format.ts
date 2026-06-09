export const formatTime = (seconds: number, fps: number = 30, long: boolean = true): string => {
	const hours = Math.floor(seconds / 3600);
	const minutes = Math.floor((seconds % 3600) / 60);
	const secs = Math.floor(seconds % 60);
	const ms = Math.floor((seconds % 1) * 1000);
	const frame = Math.floor(seconds * fps);

	const short = `${hours.toString().padStart(2, "0")}:${minutes.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;

	return long ? `${short}:${ms.toString().padStart(3, "0")} (${frame})` : short;
};

export const formatFileSize = (bytes: number): string => {
	if (bytes === 0) {
		return "0 B";
	}

	const k = 1024;
	const sizes = ["B", "KB", "MB", "GB"];
	const i = Math.floor(Math.log(bytes) / Math.log(k));

	return `${Math.round(bytes / Math.pow(k, i))} ${sizes[i]}`;
};

export const calculateEstimatedSize = (bitrate: number, duration: number): number => {
	return (bitrate / 8) * duration;
};
