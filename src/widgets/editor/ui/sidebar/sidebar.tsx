import { useHotkey } from "@tanstack/react-hotkeys";
import { useState } from "preact/hooks";

import { ThemeSwitch } from "@/features/editor/theme-switch";
import { Sheet, SheetContent, SheetHeader, SheetTrigger } from "@/shared/ui";
import { SidebarIcon } from "@/shared/ui/icons";

import { Footer } from "./footer";
import { List } from "./list";

export const Sidebar = () => {
	const [isOpen, setIsOpen] = useState(false);

	const handleToggle = () => {
		setIsOpen((p) => !p);
	};

	useHotkey("S", handleToggle);

	return (
		<Sheet open={isOpen} onOpenChange={setIsOpen}>
			<div class="bg-accent-inverted border-secondary absolute top-0 right-0 flex gap-1 rounded-bl-xl border-b border-l p-1 opacity-80 duration-100 ease-out hover:opacity-100">
				<ThemeSwitch class="button secondary icon size-9 rounded-lg" iconClass="size-6" />
				<SheetTrigger
					render={
						<button class="button secondary icon size-9 rounded-lg" title="Open Sidebar ( S )">
							<SidebarIcon class="size-8" />
						</button>
					}
				/>
			</div>
			<SheetContent>
				<SheetHeader class="select-none">Segments</SheetHeader>
				<List />
				<Footer />
			</SheetContent>
		</Sheet>
	);
};
