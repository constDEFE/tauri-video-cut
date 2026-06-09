import type { Numberish } from "../types/util";
import type { ComponentProps } from "preact";

type Props = ComponentProps<"input"> & {
	value: Numberish;
	min: Numberish;
	max: Numberish;
};

const percent = (v: Numberish = 0, min: Numberish = 0, max: Numberish = 0) => {
	const [vNum, minNum, maxNum] = [v, min, max].map(Number) as [number, number, number];
	const percentValue = Math.round(((vNum - minNum) / (maxNum - minNum)) * 100) || 0;

	return `${percentValue}%`;
};

export const Slider = ({ value, max, min, step, ...props }: Props) => (
	<input
		type="range"
		min={min}
		max={max}
		value={value}
		step={step}
		style={{ "--progress-value": percent(value, min, max) }}
		{...props}
	/>
);
