import { ColorResolvable, EmbedBuilder, EmbedData } from 'discord.js'

function parseColor(input: string | undefined): ColorResolvable | undefined {
	if (!input) return undefined
	const v = input.trim()
	// Allow formats: "#RRGGBB", "0xRRGGBB", decimal number, or named color
	if (/^#?[0-9a-fA-F]{6}$/.test(v)) {
		const hex = v.startsWith('#') ? v.slice(1) : v
		return parseInt(hex, 16)
	}
	if (/^0x[0-9a-fA-F]{6}$/.test(v)) return parseInt(v, 16) as unknown as ColorResolvable
	const n = Number(v)
	if (Number.isFinite(n) && n >= 0) return n as unknown as ColorResolvable
	// Fallback: let discord.js resolve named colors, if valid
	return v as unknown as ColorResolvable
}

/**
 * Creates an EmbedBuilder pre-configured with default styling from environment variables.
 *
 * Supported env variables:
 * - EMBED_COLOR: hex (e.g. #00FF88), 0x-prefixed, decimal, or named color
 * - EMBED_AUTHOR_NAME, EMBED_AUTHOR_ICON_URL
 * - EMBED_FOOTER_TEXT, EMBED_FOOTER_ICON_URL
 * - EMBED_THUMBNAIL_URL
 * - EMBED_TIMESTAMP: 'true' | 'false' (default true)
 */
export function createDefaultEmbed(data?: EmbedData): EmbedBuilder {
	const builder = new EmbedBuilder(data ?? {})

	const color = parseColor(process.env.EMBED_COLOR)
	if (color) builder.setColor(color)

	const authorName = process.env.EMBED_AUTHOR_NAME?.trim()
	const authorIconURL = process.env.EMBED_AUTHOR_ICON_URL?.trim()
	if (authorName) builder.setAuthor({ name: authorName, iconURL: authorIconURL || undefined })

	const footerText = process.env.EMBED_FOOTER_TEXT?.trim()
	const footerIconURL = process.env.EMBED_FOOTER_ICON_URL?.trim()
	if (footerText) builder.setFooter({ text: footerText, iconURL: footerIconURL || undefined })

	const thumbnail = process.env.EMBED_THUMBNAIL_URL?.trim()
	if (thumbnail) builder.setThumbnail(thumbnail)

	const tsPref = (process.env.EMBED_TIMESTAMP ?? 'true').toLowerCase()
	if (tsPref !== 'false') builder.setTimestamp()

	return builder
}

export type { EmbedData }
