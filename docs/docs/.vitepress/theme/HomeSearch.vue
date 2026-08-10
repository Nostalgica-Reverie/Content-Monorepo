<script setup>
function openSearch() {
	const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform)
	const event = new KeyboardEvent('keydown', {
		key: 'k',
		code: 'KeyK',
		metaKey: isMac,
		ctrlKey: !isMac,
		bubbles: true,
	})
	window.dispatchEvent(event)
}
</script>

<template>
	<div class="home-search">
		<button class="home-search-btn" @click="openSearch" aria-label="Search documentation">
			<span class="home-search-icon" aria-hidden="true">
				<svg
					width="22"
					height="22"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<circle cx="11" cy="11" r="8" />
					<path d="m21 21-4.3-4.3" />
				</svg>
			</span>
			<span class="home-search-text">Search the documentation</span>
			<span class="home-search-kbd"> <kbd>Ctrl</kbd><kbd>K</kbd> </span>
		</button>
	</div>
</template>

<style scoped>
.home-search {
	display: flex;
	justify-content: center;
	margin: 48px auto 0;
	padding: 0 24px;
	max-width: 1152px;
}

.home-search-btn {
	display: flex;
	align-items: center;
	gap: 12px;
	width: 100%;
	padding: 18px 22px;
	border: 1px solid var(--vp-c-divider);
	border-radius: 14px;
	background-color: var(--vp-c-bg-alt);
	color: var(--vp-c-text-2);
	font-size: 16px;
	cursor: pointer;
	transition:
		border-color 0.25s,
		background-color 0.25s;
}

.home-search-btn:hover {
	border-color: var(--vp-c-brand-1);
	background-color: var(--vp-c-bg-soft);
}

.home-search-icon {
	display: flex;
	color: var(--vp-c-brand-1);
}

.home-search-text {
	flex: 1;
	text-align: left;
}

.home-search-kbd {
	display: flex;
	gap: 4px;
}

.home-search-kbd kbd {
	display: inline-block;
	padding: 3px 7px;
	border: 1px solid var(--vp-c-divider);
	border-radius: 4px;
	background-color: var(--vp-c-bg);
	font-size: 12px;
	font-family: var(--vp-font-family-base);
	color: var(--vp-c-text-3);
}
</style>
