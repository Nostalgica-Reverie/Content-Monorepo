export function capitalizeFirstChar(name: string): string {
	if (!name) return ''
	return name[0].toUpperCase() + name.slice(1)
}
