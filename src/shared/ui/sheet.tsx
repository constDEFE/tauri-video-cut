import { Dialog as SheetPrimitive } from "@base-ui/react/dialog";

import { cn } from "../utils";

import type { ComponentProps } from "preact";

const CN = {
	overlay:
		"fixed inset-0 z-50 bg-black/50 transition-opacity duration-150 data-ending-style:opacity-0 data-starting-style:opacity-0 supports-backdrop-filter:backdrop-blur-xs",
	content:
		"fixed z-50 flex flex-col bg-accent-inverted shadow-lg transition duration-200 ease-in-out data-ending-style:opacity-0 data-starting-style:opacity-0 inset-y-0 right-0 h-full border-l border-secondary data-ending-style:translate-x-[2.5rem] data-starting-style:translate-x-[2.5rem] w-80",
	header: "flex flex-col gap-0.5 p-4 text-accent text-lg font-bold",
	footer: "mt-auto",
	desc: "text-sm text-text"
};

type RootProps = SheetPrimitive.Root.Props;

const Sheet = (props: RootProps) => {
	return <SheetPrimitive.Root data-slot="sheet" {...props} />;
};

type TriggerProps = SheetPrimitive.Trigger.Props;

const SheetTrigger = (props: TriggerProps) => {
	return <SheetPrimitive.Trigger data-slot="sheet-trigger" {...props} />;
};

type PortalProps = SheetPrimitive.Portal.Props;

const SheetPortal = (props: PortalProps) => {
	return <SheetPrimitive.Portal data-slot="sheet-portal" {...props} />;
};

type OverlayProps = SheetPrimitive.Backdrop.Props;

const SheetOverlay = ({ class: className, ...props }: OverlayProps) => {
	return <SheetPrimitive.Backdrop data-slot="sheet-overlay" className={cn(CN.overlay, className)} {...props} />;
};

type ContentProps = SheetPrimitive.Popup.Props & {
	keepMounted?: boolean;
};

const SheetContent = ({ class: className, children, keepMounted = true, ...props }: ContentProps) => (
	<SheetPortal keepMounted={keepMounted}>
		<SheetOverlay />
		<SheetPrimitive.Popup data-slot="sheet-content" data-side="right" className={cn(CN.content, className)} {...props}>
			{children}
		</SheetPrimitive.Popup>
	</SheetPortal>
);

const SheetHeader = ({ class: className, ...props }: ComponentProps<"div">) => {
	return <div data-slot="sheet-header" className={cn(CN.header, className)} {...props} />;
};

const SheetFooter = ({ class: className, ...props }: ComponentProps<"div">) => {
	return <div data-slot="sheet-footer" className={cn(CN.footer, className)} {...props} />;
};

export { Sheet, SheetContent, SheetFooter, SheetHeader, SheetTrigger };
