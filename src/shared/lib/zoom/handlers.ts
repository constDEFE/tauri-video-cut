const ZOOM_STEPS = [0.25, 0.333333, 0.5, 0.666666, 0.75, 0.9, 1, 1.1, 1.25, 1.5, 1.75, 2, 2.5, 3, 3.5, 4, 4.5, 5];

const ZOOM_STEP_TO_IDX_MAP = ZOOM_STEPS.reduce<Record<number, number>>((acc, v, idx) => {
	acc[v] = idx;

	return acc;
}, {});

export const zoomIn = () => {
	const currentStepIdx = ZOOM_STEP_TO_IDX_MAP[window.__CONFIG__.zoomScale];

	let nextStep;
	if (typeof currentStepIdx === "undefined") nextStep = 1;
	else nextStep = ZOOM_STEPS[currentStepIdx + 1] || 1;

	window.__CONFIG__.zoomScale = nextStep;
	window.dispatchEvent(new CustomEvent("zoomUpdate", { detail: window.__CONFIG__.zoomScale }));
};

export const zoomOut = () => {
	const currentStepIdx = ZOOM_STEP_TO_IDX_MAP[window.__CONFIG__.zoomScale];

	let nextStep;
	if (typeof currentStepIdx === "undefined") nextStep = 1;
	else nextStep = ZOOM_STEPS[currentStepIdx - 1] || 1;

	window.__CONFIG__.zoomScale = nextStep;
	window.dispatchEvent(new CustomEvent("zoomUpdate", { detail: window.__CONFIG__.zoomScale }));
};
