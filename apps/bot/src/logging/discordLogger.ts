// src/logging/discordLogger.ts
import { Client, ThreadChannel } from 'discord.js'

import { Logger } from './logger'

export async function createDiscordLogger(client: Client, channelId: string): Promise<Logger> {
	const channel = await client.channels.fetch(channelId)

	if (!channel || !channel.isTextBased()) {
		throw new Error(`Logging channel ${channelId} is not a text channel.`)
	}

	const logChannel = channel as ThreadChannel

	return {
		info(msg: string) {
			logChannel
				.send({
					content: msg,
					allowedMentions: {
						parse: [],
					},
				})
				.catch(() => null)
		},

		warn(msg: string) {
			logChannel
				.send({
					content: msg,
					allowedMentions: {
						parse: [],
					},
				})
				.catch(() => null)
		},

		error(msg: string | Error) {
			const text = msg instanceof Error ? msg.stack || msg.message : msg
			console.error(`[ERROR] ${text}`)
			logChannel.send(`**Unknown Error:** ${text}`).catch(() => null)
		},
	}
}
