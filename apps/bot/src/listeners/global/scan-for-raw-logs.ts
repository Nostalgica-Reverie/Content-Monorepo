import { DONT_UPLOAD_RAW_LOGS } from '@/data/misc'
import { CreateListener } from '@/types'
import { isInCommunitySupportThread } from '@/utils'

function isMinecraftLogFile(filename: string): boolean {
	const normalized = filename.toLowerCase()

	const exactMatches = new Set(['message.txt', 'latest.log', 'debug.log', 'launcher_log.txt'])

	if (exactMatches.has(normalized)) {
		return true
	}

	// Match crash reports like crash-2025-09-21_17.34.56-client.txt
	return /^crash-\d{4}-\d{2}-\d{2}_\d{2}\.\d{2}\.\d{2}-(client|server)\.txt$/.test(normalized)
}

export const scanForRawLogs: CreateListener = {
	id: 'forum:community-support:scan-for-raw-logs',
	event: 'create',
	description: 'Scans community support message attachments for raw logs',
	priority: 0,
	filter: { allowBots: false, allowDMs: false },
	match: async (ctx) =>
		isInCommunitySupportThread(ctx.message) &&
		ctx.message.attachments.some((attachment) => isMinecraftLogFile(attachment.name)),
	handle: async (ctx) => {
		ctx.message.attachments.forEach((attachment) => {
			if (attachment.name.length === 0) return
			if (isMinecraftLogFile(attachment.name)) {
				ctx.message.reply(DONT_UPLOAD_RAW_LOGS)
			}
		})
	},
}
