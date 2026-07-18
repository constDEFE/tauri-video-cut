import { useHotkey } from "@tanstack/react-hotkeys";

import { useVideoStore } from "@/entities/video";
import { useNavigate } from "@/shared/lib/router";

export const ExportButton = () => {
	const resetPlayer = useVideoStore((s) => s.actions.player.reset);
	const navigate = useNavigate();

	const handleClick = () => {
		navigate("/export");
		resetPlayer();
	};

	useHotkey("Control+S", handleClick);

	return (
		<button class="button h-9 rounded font-medium" onClick={handleClick} title="Export Created Segments ( Ctrl+S )">
			Export
		</button>
	);
};
