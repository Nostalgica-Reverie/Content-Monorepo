import { ThreadChannel } from 'discord.js'

import { SOLVED_REMINDER_TEXT } from '@/data/forum'
import { MessageListener } from '@/types'
import {
	isByThreadOP,
	isInCommunitySupportThread,
	isThreadSolved,
	isThreadStarterMessage,
} from '@/utils'

const solvedPhrases = [
	/\b(thanks|thank you|ty|tysm|appreciate it)\b/i,
	/\b(it works( now)?|fixed( now)?|resolved|issue resolved)\b/i,
	/\bproblem( is)? (gone|solved|fixed)\b/i,
]

function contentIndicatesSolved(content: string | null | undefined): boolean {
	const text = content ?? ''
	return solvedPhrases.some((r) => r.test(text))
}

export const remindSolvedCreate: MessageListener = {
	id: 'forum:community-support:op-solved-reminder:create',
	event: 'create',
	description: 'When OP posts a solved-like message, remind them to use /solved (create).',
	priority: 5,
	filter: { allowBots: false, allowDMs: false },
	match: async (ctx: { message: any }) => {
		if (!('message' in ctx)) return false
		const { message } = ctx
		const debug = process.env.DEBUG_FORUM_LISTENERS === '1'
		if (!isInCommunitySupportThread(message)) {
			debug &&
				console.log('[solved:create] not in forum thread', {
					guildId: message.guildId,
					channelId: message.channelId,
				})
			return false
		}
		if (!(await isByThreadOP(message))) {
			debug && console.log('[solved:create] not by OP or starter fetch failed')
			return false
		}
		if (await isThreadSolved(message)) return false
		if (await isThreadStarterMessage(message)) return false
		const ok = contentIndicatesSolved(message.content)
		debug && console.log('[solved:create] content check', { content: message.content, ok })
		return ok
	},
	handle: async (ctx: { now: any; message: { reply: (arg0: string) => any } }) => {
		if (!('message' in ctx)) return
		console.log(ctx.now)
		await ctx.message.reply(SOLVED_REMINDER_TEXT)
	},
}

export const remindSolvedUpdate: MessageListener = {
	id: 'forum:community-support:op-solved-reminder:update',
	event: 'update',
	description: 'When OP edits a message to indicate solved, remind them to use /solved (update).',
	priority: 5,
	filter: { allowBots: false, allowDMs: false },
	match: async (ctx: { newMessage: any }) => {
		if (!('newMessage' in ctx)) return false
		const { newMessage } = ctx
		if (!('channel' in newMessage)) return false
		if (await isThreadStarterMessage(newMessage)) return false
		// PartialMessage may lack content or author; bail if unknown
		const content = (newMessage as any).content as string | undefined
		const authorId = (newMessage as any).author?.id as string | undefined
		if (!content || !authorId) return false
		const debug = process.env.DEBUG_FORUM_LISTENERS === '1'
		if (!isInCommunitySupportThread(newMessage as any)) {
			debug && console.log('[solved:update] not in forum thread')
			return false
		}
		// Fetch starter to validate OP; we need a Message for isByThreadOP, so cast and rely on cache
		const ok = contentIndicatesSolved(content) && (await isByThreadOP(newMessage as any))
		debug && console.log('[solved:update] content+op check', { content, ok })
		return ok
	},
	handle: async (ctx: { newMessage: any }) => {
		if (!('newMessage' in ctx)) return
		const ch = (ctx.newMessage as any).channel as ThreadChannel
		await ch.send(SOLVED_REMINDER_TEXT)
	},
}
