/*---------------------------------------------------------------------------------------------
 *  Copyright (c) Microsoft Corporation. All rights reserved.
 *  Licensed under the MIT License. See License.txt in the project root for license information.
 *--------------------------------------------------------------------------------------------*/

import { IProductService } from '../../product/common/productService.js';
import { AbstractSignService, IVsdaValidator } from '../common/abstractSignService.js';
import { ISignService } from '../common/sign.js';

export class SignService extends AbstractSignService implements ISignService {
	constructor(@IProductService _productService: IProductService) {
		super();
	}
	protected override getValidator(): Promise<IVsdaValidator> {
		throw new Error('error loading vsda');
	}

	protected override signValue(_arg: string): Promise<string> {
		throw new Error('error loading vsda');
	}
}
