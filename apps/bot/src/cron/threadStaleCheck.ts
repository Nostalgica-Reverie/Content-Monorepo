import { CronJob } from 'cron'
import type { Client } from 'discord.js'

import { createDefaultEmbed } from '@/utils'

export function startThreadStaleCheckCron(client: Client) {
	const debug = process.env.CRON_DEBUG_MODE === '1'

	// Auto-starts (constructor start=true); no handle needed.
	new CronJob(
		debug ? '*/10 * * * * *' : '0 * * * *', // run every hour on prod, every 10 sec on debug
		async function () {
			console.log('[Cron][ThreadStaleCheck] Checking inactive threads...')

			for (const [, guild] of client.guilds.cache) {
				try {
					const active = await guild.channels.fetchActiveThreads()

					for (const [, thread] of active.threads) {
						// Ignore threads that are not in #community-support
						if (thread.parentId !== process.env.COMMUNITY_SUPPORT_FORUM_ID) continue
						// Ignore threads that are pinned
						if (thread.flags.has('Pinned')) {
							debug &&
								console.log(
									`[Debug][Cron][ThreadStaleCheck] Skipping thread ${thread.id} is pinned`,
								)
							continue
						}
						// Ignore threads that are already solved
						if (thread.appliedTags.includes(process.env.COMMUNITY_SUPPORT_FORUM_SOLVED_TAG_ID!)) {
							debug &&
								console.log(
									`[Debug][Cron][ThreadStaleCheck] Skipping thread ${thread.id} is solved`,
								)
							continue
						}

						// Last message
						const lastMessage = thread.lastMessageId
							? await thread.messages.fetch(thread.lastMessageId).catch(() => null)
							: null

						const inactiveFor = lastMessage ? Date.now() - lastMessage.createdTimestamp : Infinity

						// thresholds
						const warnThreshold = debug ? 1000 * 15 : 1000 * 60 * 60 * 24 * 3 // 3 days (prod), 15 sec (debug)
						const archiveThreshold = debug ? 1000 * 30 : 1000 * 60 * 60 * 24 * 5 // 5 days (prod), 30 sec (debug)

						if (inactiveFor > archiveThreshold) {
							debug &&
								console.log(
									`[Debug][Cron][ThreadStaleCheck] Marking inactive thread as solved: ${thread.name}`,
								)

							const embed = createDefaultEmbed({
								title: ':white_check_mark: Thread archived',
								description: 'This thread was resolved automatically after inactivity.',
							})

							await thread.send({ embeds: [embed] })
							await thread
								.edit({
									archived: true,
									appliedTags: [process.env.COMMUNITY_SUPPORT_FORUM_SOLVED_TAG_ID!],
								})
								.catch(() => {})
						} else if (lastMessage && inactiveFor > warnThreshold) {
							const unixTs = Math.floor(lastMessage.createdTimestamp / 1000)
							const warningMsg = `-# ⚠️ Last message in this thread was sent <t:${unixTs}:R>. It will be automatically archived soon.`

							// Prevent spam: don’t re-send warning if already sent
							const recentMsgs = await thread.messages.fetch({ limit: 10 })
							const alreadyWarned = recentMsgs.some(
								(m) =>
									m.author.id === client.user?.id && m.content.includes('automatically archived'),
							)

							if (!alreadyWarned) {
								debug &&
									console.log(`[Debug][Cron] Sending stale warning to the thread ${thread.id}`)
								await thread.send(warningMsg).catch(() => {})
							}
						}
					}
				} catch (err) {
					console.warn(`[Cron] Failed to check guild ${guild.id}`, err)
				}
			}
		},
		null,
		true,
		'America/Los_Angeles',
	)
}
