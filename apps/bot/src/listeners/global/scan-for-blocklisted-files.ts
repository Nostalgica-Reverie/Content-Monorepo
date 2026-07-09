import { createHash } from 'crypto'
import { TextChannel } from 'discord.js'

import { BLOCKLISTED_FILE_EXTENSIONS } from '@/data/misc'
import { error } from '@/logging/logger'
import { CreateListener } from '@/types'
import { toHumanFileSize } from '@/utils'

type HashResult = {
	sha512: string
	sha256: string
	sha1: string
}

export async function hashAttachment(url: string): Promise<HashResult> {
	const res = await fetch(url)

	if (!res.ok || !res.body) {
		error(`:warning: Failed to download attachment (\`${url}\`) for processing.`)
	}

	const sha512 = createHash('sha512')
	const sha256 = createHash('sha256')
	const sha1 = createHash('sha1')

	const stream = res.body as unknown as AsyncIterable<Uint8Array>

	for await (const chunk of stream) {
		sha512.update(chunk)
		sha256.update(chunk)
		sha1.update(chunk)
	}

	return {
		sha512: sha512.digest('hex'),
		sha256: sha256.digest('hex'),
		sha1: sha1.digest('hex'),
	}
}

function getFileExtension(filename: string): string | null {
	const parts = filename.split('.')
	return parts.length > 1 ? parts.pop() || null : null
}

export const scanForBlocklistedFiles: CreateListener = {
	id: 'global:scan-for-blocklisted-files',
	event: 'create',
	description: 'Scans message attachments for blocklisted file extensions',
	priority: 0,
	filter: { allowBots: false, allowDMs: false },
	match: async () => true,
	handle: async (ctx) => {
		if (!ctx.message.guild) return
		if (!ctx.message.author) return
		if (!ctx.message.attachments) return

		for (const attachment of ctx.message.attachments.values()) {
			if (!attachment.name) continue

			const ext = getFileExtension(attachment.name)
			if (!ext) continue

			if (BLOCKLISTED_FILE_EXTENSIONS.includes(ext)) {
				let hashes: HashResult | null = null

				try {
					hashes = await hashAttachment(attachment.url)
				} catch (err) {
					error(
						`:warning: Failed to process checksums for attachment (\`${attachment.url}\`).\n\nError: \`\`\`${err}\`\`\``,
					)
				}

				await ctx.message.delete()

				const channel = await ctx.client.channels.fetch(process.env.MOD_CHANNEL_ID!)

				if (channel && channel.isTextBased()) {
					await (channel as TextChannel).send(
						[
							`⚠️ User ${ctx.message.author} (\`${ctx.message.author.username}\`, ID: ${ctx.message.author.id}) attempted to sent a blacklisted file type in ${ctx.message.channel}.`,
							`> Filename: \`${attachment.name}\``,
							`> Size: ${toHumanFileSize(attachment.size)}`,
							hashes && ` `,
							hashes && `> SHA-512: \`\`\`${hashes.sha512}\`\`\``,
							hashes && `> SHA-256: \`\`\`${hashes.sha256}\`\`\``,
							hashes && `> SHA-1: \`\`\`${hashes.sha1}\`\`\``,
							hashes && ` `,
							hashes && `> VirusTotal: <https://virustotal.com/gui/file/${hashes.sha256}>`,
						].join('\n'),
					)
				}
			}
		}
	},
}
