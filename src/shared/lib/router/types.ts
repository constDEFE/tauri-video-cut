export type NavigateFn = (path: string, options?: NavigateOptions) => void;

export type NavigateOptions = {
	state?: any;
};

export type Route = {
	path: string;
	state?: any;
};
