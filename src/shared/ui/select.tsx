"use client";

import { Select as SelectPrimitive } from "@base-ui/react/select";

import { cn } from "../utils";
import { CheckIcon, ChevronDownIcon } from "./icons";

const CN = {
	group: "scroll-my-1 p-1",
	value: "flex flex-1 text-left",
	positioner: "isolate z-50",
	label: "text-secondary px-1.5 py-1 text-sm select-none",
	itemText: "flex flex-1 text-accent shrink-0 gap-2 whitespace-nowrap",
	itemIndicator: "pointer-events-none absolute right-2 flex size-4 items-center justify-center",
	item: "hover:bg-accent-inverted focus:bg-accent-inverted outline outline-surface hover:outline-accent focus:outline-accent duration-100 ease-out hover:text-accent focus:text-accent relative flex w-full cursor-default items-center gap-1.5 rounded-md py-1 pr-8 pl-1.5 text-sm select-none [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 *:[span]:last:flex *:[span]:last:items-center *:[span]:last:gap-2",
	trigger:
		"border-secondary data-placeholder:text-secondary flex w-fit items-center justify-between gap-1.5 rounded-lg border bg-accent-inverted py-1.5 pr-1.5 pl-2 text-sm whitespace-nowrap transition-colors outline-none select-none focus-visible:border-accent disabled:cursor-not-allowed *:data-[slot=select-value]:line-clamp-1 *:data-[slot=select-value]:flex *:data-[slot=select-value]:items-center *:data-[slot=select-value]:gap-1.5 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
	content:
		"bg-surface text-accent relative isolate z-50 max-h-(--available-height) w-(--anchor-width) min-w-36 origin-(--transform-origin) overflow-x-hidden overflow-y-auto rounded-lg shadow-md ring-1 ring-secondary duration-100"
};

const Select = SelectPrimitive.Root;

type SelectGroupProps = SelectPrimitive.Group.Props;

const SelectGroup = ({ className, ...props }: SelectGroupProps) => (
	<SelectPrimitive.Group data-slot="select-group" className={cn(CN.group, className)} {...props} />
);

type SelectValueProps = SelectPrimitive.Value.Props;

const SelectValue = ({ className, ...props }: SelectValueProps) => (
	<SelectPrimitive.Value data-slot="select-value" className={cn(CN.value, className)} {...props} />
);

type SelectTriggerProps = SelectPrimitive.Trigger.Props & {
	size?: "sm" | "default";
};

const SelectTrigger = ({ className, size = "default", children, ...props }: SelectTriggerProps) => (
	<SelectPrimitive.Trigger data-slot="select-trigger" data-size={size} className={cn(CN.trigger, className)} {...props}>
		{children}
		<SelectPrimitive.Icon render={<ChevronDownIcon className="text-text pointer-events-none size-4" />} />
	</SelectPrimitive.Trigger>
);

type SelectContentProps = SelectPrimitive.Popup.Props &
	Pick<SelectPrimitive.Positioner.Props, "align" | "alignOffset" | "side" | "sideOffset" | "alignItemWithTrigger">;

const SelectContent = ({
	className,
	children,
	side,
	sideOffset = 10,
	align = "center",
	alignOffset = 0,
	alignItemWithTrigger = true,
	...props
}: SelectContentProps) => (
	<SelectPrimitive.Portal>
		<SelectPrimitive.Positioner
			side={side}
			sideOffset={sideOffset}
			align={align}
			alignOffset={alignOffset}
			alignItemWithTrigger={alignItemWithTrigger}
			className={CN.positioner}
		>
			<SelectPrimitive.Popup
				data-slot="select-content"
				data-align-trigger={alignItemWithTrigger}
				className={cn(CN.content, className)}
				{...props}
			>
				<SelectPrimitive.List>{children}</SelectPrimitive.List>
			</SelectPrimitive.Popup>
		</SelectPrimitive.Positioner>
	</SelectPrimitive.Portal>
);

const SelectLabel = ({ className, ...props }: SelectPrimitive.GroupLabel.Props) => (
	<SelectPrimitive.GroupLabel data-slot="select-label" className={cn(CN.label, className)} {...props} />
);

const SelectItem = ({ className, children, ...props }: SelectPrimitive.Item.Props) => (
	<SelectPrimitive.Item data-slot="select-item" className={cn(CN.item, className)} {...props}>
		<SelectPrimitive.ItemText className={CN.itemText}>{children}</SelectPrimitive.ItemText>
		<SelectPrimitive.ItemIndicator
			render={
				<span className={CN.itemIndicator}>
					<CheckIcon />
				</span>
			}
		/>
	</SelectPrimitive.Item>
);

export { Select, SelectGroup, SelectContent, SelectItem, SelectLabel, SelectTrigger, SelectValue };
