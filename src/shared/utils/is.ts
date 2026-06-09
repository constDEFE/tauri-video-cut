/* eslint-disable no-control-regex */
const illegalCharsRegex = /[<>:"/\\|?*\x00-\x1F]/;

export const isValidPrefix = (str: string) => {
	/* max_filename - extension */
	if (!str || str.length > 255 - 5) {
		return false;
	}

	if (illegalCharsRegex.test(str)) {
		return false;
	}

	return true;
};
