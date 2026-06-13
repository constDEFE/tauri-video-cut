import { memo, useState } from "preact/compat";

import { useVideoStore } from "@/entities/video";
import { Popover, PopoverContent } from "@/shared/ui";

import { Select } from "./select";
import { Trigger } from "./trigger";

type Props = {
	isDisabled?: boolean;
};

export const AudioSelector = memo(({ isDisabled }: Props) => {
	const [isOpen, setIsOpen] = useState(false);
	const hasTracks = useVideoStore((s) => !!s.state.metadata?.audio_tracks.length);

	const handleOpenChange = (isOpen: boolean) => {
		setIsOpen(isOpen);
	};

	const handleSelect = () => {
		setIsOpen(false);
	};

	return (
		<Popover open={isOpen} onOpenChange={handleOpenChange}>
			<Trigger isDisabled={isDisabled || !hasTracks} />
			<PopoverContent align="start" sideOffset={8}>
				<Select onSelect={handleSelect} />
			</PopoverContent>
		</Popover>
	);
});
