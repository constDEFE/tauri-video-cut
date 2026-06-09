export const getFileNameWithoutExtension = (path: string | null, fallback: string = "file") => {
	if (!path) {
		return fallback;
	}

	const fileName = path.split(/[\\/]/).pop() || fallback;

	return fileName.replace(/\.[^.]+$/, "");
};
