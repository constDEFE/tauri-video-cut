import { useTheme } from "@/shared/lib/theme";
import { MoonIcon, SunIcon } from "@/shared/ui/icons";

type Props = {
	class?: string;
	iconClass?: string;
};

export const ThemeSwitch = ({ class: cn, iconClass }: Props) => {
	const { theme, toggleTheme } = useTheme();

	const isDarkTheme = theme === "dark";

	return (
		<button onClick={toggleTheme} class={cn} title={isDarkTheme ? "Switch to light mode" : "Switch to dark mode"}>
			{isDarkTheme ? <MoonIcon class={iconClass} /> : <SunIcon class={iconClass} />}
		</button>
	);
};
