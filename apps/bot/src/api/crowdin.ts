export type CrowdinOAuthToken = {
	token_type: string
	access_token: string
	refresh_token?: string
	expires_in: number
}

type ReportCreate = { data: { id?: string; identifier?: string } }
type ReportStatus = { data: { status: 'created' | 'inProgress' | 'finished' | 'failed' } }
type ReportDownload = { data: { url: string } }

export class CrowdinOauthHelper {
	private readonly clientId: string
	private readonly clientSecret: string
	private readonly redirectUri: string
	private readonly scopes: string
	private readonly apiBase: string
	private readonly reportCache = new Map<string, { expiresAt: number; rows: any[] }>()
	private readonly reportInFlight = new Map<string, Promise<any[]>>()

	constructor(opts: {
		baseUrl: string
		clientId: string
		clientSecret: string
		scopes?: string
		apiBase?: string
	}) {
		this.clientId = opts.clientId
		this.clientSecret = opts.clientSecret
		this.redirectUri = `${opts.baseUrl}/callback/crowdin`
		this.scopes = opts.scopes ?? 'project'
		this.apiBase = opts.apiBase ?? 'https://api.crowdin.com/api/v2'
	}

	buildAuthorizeUrl(state: string) {
		const url = new URL('https://accounts.crowdin.com/oauth/authorize')
		url.searchParams.set('client_id', this.clientId)
		url.searchParams.set('response_type', 'code')
		url.searchParams.set('scope', this.scopes)
		url.searchParams.set('state', state)
		url.searchParams.set('redirect_uri', this.redirectUri)
		return url.toString()
	}

	async exchangeCodeForToken(code: string): Promise<CrowdinOAuthToken> {
		const res = await fetch('https://accounts.crowdin.com/oauth/token', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({
				grant_type: 'authorization_code',
				client_id: this.clientId,
				client_secret: this.clientSecret,
				redirect_uri: this.redirectUri,
				code,
			}),
		})
		if (!res.ok) {
			const body = await res.text()
			throw new Error(`Crowdin token exchange failed: ${res.status} ${body}`)
		}
		return (await res.json()) as CrowdinOAuthToken
	}

	private async crowdinGet<T>(endpoint: string, accessToken: string): Promise<T> {
		const url = `${this.apiBase}${endpoint}`
		const res = await fetch(url, { headers: { Authorization: `Bearer ${accessToken}` } })
		if (!res.ok) {
			const body = await res.text()
			throw new Error(`Crowdin API error ${res.status} at GET ${endpoint}: ${body}`)
		}
		return (await res.json()) as T
	}

	private async crowdinPost<T>(endpoint: string, accessToken: string, body: any): Promise<T> {
		const url = `${this.apiBase}${endpoint}`
		const res = await fetch(url, {
			method: 'POST',
			headers: { Authorization: `Bearer ${accessToken}`, 'Content-Type': 'application/json' },
			body: JSON.stringify(body),
		})
		if (!res.ok) {
			const respBody = await res.text()
			throw new Error(`Crowdin API error ${res.status} at POST ${endpoint}: ${respBody}`)
		}
		return (await res.json()) as T
	}

	async getCurrentUser(accessToken: string): Promise<{ id: number }> {
		const res = await this.crowdinGet<{ data: { id: number } }>(`/user`, accessToken)
		return res.data
	}

	async getProjectMember(
		projectId: string | number,
		memberId: string | number,
		serviceToken: string,
	): Promise<{
		id: number
		username?: string
		fullName?: string | null
		role?: string
		roles?: Array<{ name?: string }>
	} | null> {
		try {
			const res = await this.crowdinGet<{
				data: {
					id: number
					username?: string
					fullName?: string | null
					role?: string
					roles?: Array<{ name?: string }>
				}
			}>(`/projects/${projectId}/members/${memberId}`, serviceToken)
			return res?.data ?? null
		} catch {
			return null
		}
	}

	async hasProofreaderRole(
		projectId: string | number,
		userId: string | number,
		serviceToken: string,
	): Promise<boolean> {
		try {
			const member = await this.getProjectMember(projectId, userId, serviceToken)
			if (!member) return false

			// Check primary role
			const primaryRole = (member.role ?? '').toLowerCase()
			if (primaryRole === 'proofreader' || primaryRole === 'owner' || primaryRole === 'manager') {
				return true
			}

			// Check roles array for proofreader or language_coordinator
			const roleNames = (member.roles ?? []).map((r) => (r.name ?? '').toLowerCase())
			return roleNames.includes('proofreader') || roleNames.includes('language_coordinator')
		} catch (error) {
			console.error('[Crowdin] Error checking proofreader role:', error)
			return false
		}
	}

	async generateTopMembersReport(projectId: string | number, accessToken: string): Promise<string> {
		const create = await this.crowdinPost<ReportCreate>(
			`/projects/${projectId}/reports`,
			accessToken,
			{
				name: 'top-members',
				schema: {
					unit: 'strings',
					format: 'json',
					dateFrom: new Date(0).toISOString(),
					dateTo: new Date().toISOString(),
				},
			},
		)
		const reportId = create.data.id ?? create.data.identifier
		for (let i = 0; i < 30; i++) {
			await new Promise((r) => setTimeout(r, 1000))
			const status = await this.crowdinGet<ReportStatus>(
				`/projects/${projectId}/reports/${reportId}`,
				accessToken,
			)
			if (status.data.status === 'finished') break
			if (status.data.status === 'failed') throw new Error('Report generation failed')
			if (i === 29) throw new Error('Report generation timed out')
		}
		const dl = await this.crowdinGet<ReportDownload>(
			`/projects/${projectId}/reports/${reportId}/download`,
			accessToken,
		)
		return dl.data.url
	}

	private async getTopMembersRows(projectId: string | number, accessToken: string): Promise<any[]> {
		const key = String(projectId)
		const now = Date.now()
		const cached = this.reportCache.get(key)
		if (cached && cached.expiresAt > now) return cached.rows
		const inflight = this.reportInFlight.get(key)
		if (inflight) return inflight
		const promise = (async () => {
			const url = await this.generateTopMembersReport(projectId, accessToken)
			const res = await fetch(url)
			if (!res.ok) throw new Error(`Crowdin report download failed: ${res.status}`)
			const data = await res.json().catch(() => null)
			const rows: any[] = Array.isArray(data?.data) ? data.data : []
			this.reportCache.set(key, { expiresAt: now + 1 * 60 * 1000, rows })
			return rows
		})()
		this.reportInFlight.set(key, promise)
		try {
			const rows = await promise
			return rows
		} finally {
			this.reportInFlight.delete(key)
		}
	}

	/**
	 * Returns the translated and approved counts for a specific user in a project
	 * using the cached "top-members" report.
	 */
	async getMemberActivity(
		projectId: string | number,
		userId: number | string,
		accessToken: string,
	): Promise<{ translated: number; approved: number }> {
		try {
			const rows = await this.getTopMembersRows(projectId, accessToken)

			// First try to find by user ID
			let found = rows.find((row: any) => {
				// The user data is nested under "user" property in the report
				const userObj = row?.user
				if (!userObj) return false
				const id = userObj.id
				if (id == null) return false
				return String(id) === String(userId)
			})

			// If not found by ID, try to get member info and match by username
			if (!found) {
				try {
					const member = await this.getProjectMember(projectId, userId, accessToken)
					if (member && member.username) {
						const targetUsername = member.username.toLowerCase()
						found = rows.find((row: any) => {
							const userObj = row?.user
							if (!userObj) return false
							const rowUsername = (userObj.username ?? '').toLowerCase()
							return rowUsername === targetUsername
						})
					}
				} catch (err) {
					console.error('[Crowdin] Error fetching member info for fallback:', err)
				}
			}

			if (!found) {
				console.debug(`[Crowdin] No activity found for user ${userId} in project ${projectId}`)
				return { translated: 0, approved: 0 }
			}

			const translated = Number(found?.translated ?? 0)
			const approved = Number(found?.approved ?? 0)
			return { translated, approved }
		} catch (error) {
			console.error('[Crowdin] Error getting member activity:', error)
			return { translated: 0, approved: 0 }
		}
	}

	async hasContributionViaReport(
		projectId: string | number,
		userId: number,
		accessToken: string,
	): Promise<boolean> {
		try {
			const activity = await this.getMemberActivity(projectId, userId, accessToken)
			const hasContribution = activity.translated > 0 || activity.approved > 0
			console.log(
				`[Crowdin] User ${userId} has ${activity.translated} translated and ${activity.approved} approved strings`,
			)
			return hasContribution
		} catch (error) {
			console.error('[Crowdin] Error checking contribution:', error)
			return false
		}
	}
}
