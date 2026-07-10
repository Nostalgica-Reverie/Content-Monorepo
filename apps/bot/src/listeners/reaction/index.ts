import { MessageReactListener } from '../../types/listeners'
import { moderatorPingReminder } from './mod-ping-reminder'

const reactionListeners: MessageReactListener[] = [moderatorPingReminder]

export default reactionListeners
