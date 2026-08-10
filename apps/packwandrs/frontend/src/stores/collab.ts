import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

import { collabConnectionKey, collabCore, collabRoleKey, type CollabModel } from '@/core/packwand'
import { onCollabOutput, onCollabParticipant, onCollabState } from '@/helpers/events'
import {
	collabFollow,
	collabHostStart,
	collabHostStop,
	collabJoin,
	collabLeave,
	collabOutput,
	collabProblems,
	collabSetGitWrite,
	collabState,
	type CollabState,
	type Participant,
	type ParticipantId,
} from '@/helpers/invoke/collab'

import { useShellStore } from './shell'
import { useWorkbenchStore } from './workbench'

export const useCollabStore = defineStore('collab', () => {
	const model = ref<CollabModel>(collabCore.init())
	const participants = ref<Participant[]>([])
	const invite = ref<string | null>(null)
	const followTarget = ref<ParticipantId | null>(null)
	const allowGitWrite = ref(false)
	let listening: Promise<void> | null = null

	const role = computed<'host' | 'guest' | null>(() => {
		const key = collabRoleKey(model.value.role)
		return key === 'host' || key === 'guest' ? key : null
	})
	const connection = computed<'disconnected' | 'connecting' | 'connected'>(() => {
		const key = collabConnectionKey(model.value.connection)
		if (key === 'connected') return 'connected'
		if (key === 'hosting' || key === 'joining') return 'connecting'
		return 'disconnected'
	})
	const isGuest = computed(() => role.value === 'guest')

	function dispatch(message: Parameters<typeof collabCore.update>[1]) {
		model.value = collabCore.update(model.value, message)
	}

	function applyState(state: CollabState) {
		const workbench = useWorkbenchStore()
		if (state.role === null) {
			dispatch(new collabCore.ConnectionLost())
		} else {
			if (role.value !== state.role) {
				dispatch(new collabCore.ConnectionLost())
				dispatch(
					state.role === 'host' ? new collabCore.HostStarted() : new collabCore.JoinStarted(),
				)
			}
			if (state.connection === 'connected') {
				dispatch(new collabCore.ConnectionEstablished())
			} else if (state.connection === 'disconnected' && state.role === 'guest') {
				dispatch(new collabCore.ConnectionLost())
			}
		}
		participants.value = state.participants
		allowGitWrite.value = state.allowGitWrite
		if (state.role === 'guest' && state.session) workbench.setRemotePack(state.session)
		if (!isGuest.value) followTarget.value = null
	}

	async function initialize() {
		if (listening) return listening
		listening = (async () => {
			const shell = useShellStore()
			await Promise.all([
				onCollabState(applyState),
				onCollabParticipant((event) => {
					if (event.participant) {
						participants.value = [
							...participants.value.filter(
								(participant) => participant.id !== event.participant!.id,
							),
							event.participant,
						]
					} else if (event.id !== null) {
						participants.value = participants.value.filter(
							(participant) => participant.id !== event.id,
						)
					}
				}),
				onCollabOutput((event) => {
					if (event.type === 'output') {
						shell.appendOutput(`[Host/${event.channel}] ${event.line}`, 'info', true)
					} else if (event.type === 'problems') {
						shell.setProblems(`Host: ${event.snapshot.source}`, event.snapshot.issues)
					} else {
						shell.applyHostJobEvent(event.event, event.payload as Record<string, unknown>)
					}
				}),
			])
			shell.$onAction(({ name, args, after }) => {
				after(() => {
					if (role.value !== 'host' || connection.value !== 'connected') return
					if (name === 'appendOutput' && !args[2]) {
						void collabOutput('output', String(args[0])).catch(() => undefined)
					} else if (name === 'setProblems') {
						void collabProblems(String(args[0]), args[1] as never[]).catch(() => undefined)
					}
				})
			})
			applyState(await collabState())
		})()
		return listening
	}

	async function host(packId: string) {
		dispatch(new collabCore.HostStarted())
		try {
			invite.value = await collabHostStart(packId, true)
			applyState(await collabState())
			return invite.value
		} catch (error) {
			dispatch(new collabCore.ConnectionLost())
			throw error
		}
	}

	async function join(value: string) {
		dispatch(new collabCore.JoinStarted())
		try {
			invite.value = null
			applyState(await collabJoin(value))
		} catch (error) {
			dispatch(new collabCore.ConnectionLost())
			throw error
		}
	}

	async function leave() {
		if (role.value === 'host') await collabHostStop()
		else await collabLeave()
		invite.value = null
		participants.value = []
		followTarget.value = null
		dispatch(new collabCore.ConnectionLost())
	}

	async function setAllowGitWrite(allow: boolean) {
		applyState(await collabSetGitWrite(allow))
	}

	async function follow(id: ParticipantId | null) {
		followTarget.value = id
		if (id !== null) await collabFollow(id)
	}

	return {
		role,
		participants,
		invite,
		connection,
		followTarget,
		allowGitWrite,
		isGuest,
		initialize,
		host,
		join,
		leave,
		setAllowGitWrite,
		follow,
	}
})
