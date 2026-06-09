import { Checkbox } from "@/shared/ui";

type Props = {
	defaultChecked: boolean;
};

export const SmartCutField = ({ defaultChecked }: Props) => (
	<div class="surface rounded-lg p-4">
		<Checkbox name="smart-cut" defaultChecked={defaultChecked}>
			Smart Cut (re-encode segments not starting with keyframes)
		</Checkbox>
	</div>
);
