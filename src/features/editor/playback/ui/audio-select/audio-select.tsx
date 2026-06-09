import { memo } from "preact/compat";

import { Popover, PopoverContent } from "@/shared/ui";

import { Select } from "./select";
import { Trigger } from "./trigger";

type Props = {
	isDisabled?: boolean;
};

export const AudioSelector = memo(({ isDisabled }: Props) => (
	<Popover>
		<Trigger isDisabled={isDisabled} />
		<PopoverContent align="start" sideOffset={8}>
			<Select />
		</PopoverContent>
	</Popover>
));
