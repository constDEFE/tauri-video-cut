"use client";

import { Popover as PopoverPrimitive } from "@base-ui/react/popover";

import { cn } from "../utils";

const CN = {
	popup:
		"rounded-lg duration-100 ease-out bg-surface text-accent ring-secondary data-ending-style:scale-[0.98] data-ending-style:opacity-0 data-starting-style:scale-[0.98] data-starting-style:opacity-0 z-50 flex w-fit-content origin-(--transform-origin) flex-col gap-2.5 rounded-lg p-0.5 text-sm shadow-md ring-1 outline-hidden duration-100"
};

type PopoverProps = PopoverPrimitive.Root.Props;

const Popover = (props: PopoverProps) => <PopoverPrimitive.Root data-slot="popover" {...props} />;

type PopoverTriggerProps = PopoverPrimitive.Trigger.Props;

const PopoverTrigger = (props: PopoverTriggerProps) => (
	<PopoverPrimitive.Trigger data-slot="popover-trigger" {...props} />
);

type PopoverContentProps = PopoverPrimitive.Popup.Props &
	Pick<PopoverPrimitive.Positioner.Props, "align" | "alignOffset" | "side" | "sideOffset">;

const PopoverContent = ({
	class: className,
	align = "center",
	alignOffset = 0,
	sideOffset = 4,
	...props
}: PopoverContentProps) => (
	<PopoverPrimitive.Portal>
		<PopoverPrimitive.Positioner
			align={align}
			alignOffset={alignOffset}
			side="top"
			sideOffset={sideOffset}
			className="isolate z-50"
		>
			<PopoverPrimitive.Popup data-slot="popover-content" className={cn(CN.popup, className)} {...props} />
		</PopoverPrimitive.Positioner>
	</PopoverPrimitive.Portal>
);

export { Popover, PopoverContent, PopoverTrigger };
