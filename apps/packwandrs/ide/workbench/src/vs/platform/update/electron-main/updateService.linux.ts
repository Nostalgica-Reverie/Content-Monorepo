/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import { IConfigurationService } from '../../configuration/common/configuration.js';
import { IEnvironmentMainService } from '../../environment/electron-main/environmentMainService.js';
import { ILifecycleMainService } from '../../lifecycle/electron-main/lifecycleMainService.js';
import { ILogService } from '../../log/common/log.js';
import { IMeteredConnectionService } from '../../meteredConnection/common/meteredConnection.js';
import { INativeHostMainService } from '../../native/electron-main/nativeHostMainService.js';
import { IProductService } from '../../product/common/productService.js';
import { IRequestService } from '../../request/common/request.js';
import { IApplicationStorageMainService } from '../../storage/electron-main/storageMainService.js';
import { ITelemetryService } from '../../telemetry/common/telemetry.js';
import { AvailableForDownload, State, UpdateType } from '../common/update.js';
import { AbstractUpdateService, createUpdateURL, IUpdateURLOptions } from './abstractUpdateService.js';

export class LinuxUpdateService extends AbstractUpdateService {

	constructor(
		@ILifecycleMainService lifecycleMainService: ILifecycleMainService,
		@IConfigurationService configurationService: IConfigurationService,
		@IEnvironmentMainService environmentMainService: IEnvironmentMainService,
		@IRequestService requestService: IRequestService,
		@ILogService logService: ILogService,
		@INativeHostMainService private readonly nativeHostMainService: INativeHostMainService,
		@IProductService productService: IProductService,
		@ITelemetryService telemetryService: ITelemetryService,
		@IApplicationStorageMainService applicationStorageMainService: IApplicationStorageMainService,
		@IMeteredConnectionService meteredConnectionService: IMeteredConnectionService,
	) {
		super(lifecycleMainService, configurationService, environmentMainService, requestService, logService, productService, telemetryService, applicationStorageMainService, meteredConnectionService, false);
	}

	protected buildUpdateFeedUrl(quality: string, _options?: IUpdateURLOptions): string {
		return createUpdateURL(this.productService, quality, process.platform, process.arch);
	}

	protected doCheckForUpdates(explicit: boolean, pendingVersion?: string): void {
		if (!this.quality) {
			return;
		}

		this.setState(State.CheckingForUpdates(explicit));

		const internalOrg = this.getInternalOrg();
		const background = !explicit && !internalOrg;
		const url = this.buildUpdateFeedUrl(this.quality, { background, internalOrg });

		this.logService.info('update#doCheckForUpdates', { url, explicit, background });

		this._isLatestVersion(url, explicit, pendingVersion)
			.then((result) => {
				if(!result) {
					this.setState(State.Idle(UpdateType.Archive));

					return Promise.resolve(null);
				}

				if(result.lastest) {
					this.setState(State.Idle(UpdateType.Archive, undefined, explicit || undefined));
				}
				else {
					this.setState(State.AvailableForDownload(result.update));
				}

				return Promise.resolve(null);
			})
			.then(undefined, (error) => {
				this.logService.error(error);

				// only show message when explicitly checking for updates
				const message: string | undefined = explicit ? (error.message || error) : undefined;

				this.setState(State.Idle(UpdateType.Archive, message));
			});
	}

	protected override async doDownloadUpdate(state: AvailableForDownload): Promise<void> {
		// Use the download URL if available as we don't currently detect the package type that was
		// installed and the website download page is more useful than the tarball generally.
		if (this.productService.downloadUrl && this.productService.downloadUrl.length > 0) {
			this.nativeHostMainService.openExternal(undefined, this.productService.downloadUrl);
		} else if (state.update.url) {
			this.nativeHostMainService.openExternal(undefined, state.update.url);
		}

		this.setState(State.Idle(UpdateType.Archive));
	}
}
