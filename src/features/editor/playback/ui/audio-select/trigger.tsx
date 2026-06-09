import { PopoverTrigger } from "@/shared/ui";
import { WaveFormIcon } from "@/shared/ui/icons";

type Props = {
	isDisabled?: boolean | undefined;
};

export const Trigger = ({ isDisabled }: Props) => (
	<PopoverTrigger
		render={
			<button disabled={isDisabled} class="button secondary icon size-9 rounded-lg" title="Open Audio Tracks">
				<WaveFormIcon class="size-6" />
			</button>
		}
	/>
);
