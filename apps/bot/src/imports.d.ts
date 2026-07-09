// Text imports via import attributes (`with { type: 'text' }`), natively
// supported by the Bun runtime.
declare module '*.txt' {
	const content: string
	export default content
}

declare module '*.html' {
	const content: string
	export default content
}
