/**
 * Decorators to allow the creation of a subset of JSON Schema from TypeScript classes
 */

// TODO(gen): annot/field indicating when a field was added/updated, for a changelog

import "npm:reflect-metadata";
import { JSONSchema7, JSONSchema7Type } from "npm:@types/json-schema";

interface PropertyMetadata {
	type?: "object" | "string" | "array" | "boolean" | "number";
	description?: string;
	required?: boolean;
	enum?: string[];
	uniqueItems?: boolean;
	default?: JSONSchema7Type;
	examples?: JSONSchema7Type[];
}

const objectProperties = Symbol("objectProperties");
type ObjectProperties = { [key: string | symbol]: PropertyMetadata };
declare global {
	namespace Reflect {
		export function getMetadata<T>(
			metadataKey: typeof objectProperties,
			target: T,
		): ObjectProperties;
		export function defineMetadata(
			metadataKey: typeof objectProperties,
			metadata: ObjectProperties,
			// deno-lint-ignore ban-types
			target: Object,
		): void;
	}
}

function _updateMetadata(data: PropertyMetadata) {
	// deno-lint-ignore ban-types
	return (target: Object, propertyKey: string | symbol): void => {
		const metadata = Reflect.getMetadata(objectProperties, target) ?? {};
		removeUndefinedValues(data);
		metadata[propertyKey] = {
			...metadata[propertyKey],
			...data,
		};
		Reflect.defineMetadata(objectProperties, metadata, target);
	};
}

export abstract class property {
	static string(description?: string) {
		return _updateMetadata({
			type: "string",
			description,
		});
	}

	static enum(values: string[], description?: string) {
		return _updateMetadata({
			type: "string",
			description,
			enum: values,
		});
	}

	static boolean(description?: string) {
		return _updateMetadata({
			type: "boolean",
			description,
		});
	}

	static number(description?: string) {
		return _updateMetadata({
			type: "number",
			description,
		});
	}

	static ref(description?: string) {
		return _updateMetadata({
			description,
		}) as <
			T extends Record<K, SchemaGenerator>,
			K extends string | number | symbol,
		>(target: T, propertyKey: K) => void;
		// ^ Enforces that property.object decorators are only applied to SchemaGenerator properties
	}

	static arrayRef(description?: string, uniqueItems = true) {
		return _updateMetadata({
			type: "array",
			description,
			uniqueItems,
		}) as <
			T extends Record<K, SchemaGenerator>,
			K extends string | number | symbol,
		>(target: T, propertyKey: K) => void;
		// ^ Enforces that property.array decorators are only applied to SchemaGenerator properties
	}

	// deno-lint-ignore ban-types
	static required(target: Object, propertyKey: string | symbol) {
		_updateMetadata({
			required: true,
		})(target, propertyKey);
	}

	static default(value: JSONSchema7Type) {
		return _updateMetadata({
			default: value,
		});
	}

	static examples(value: JSONSchema7Type[]) {
		return _updateMetadata({
			examples: value,
		});
	}
}

function err(message: string): never {
	throw new Error(message);
}

// deno-lint-ignore ban-types
function removeUndefinedValues(obj: object) {
	for (const key of Object.keys(obj) as (keyof typeof obj)[]) {
		if (obj[key] === undefined) {
			delete obj[key];
		}
	}
}

export interface SchemaGenerator {
	readonly schema: JSONSchema7;
}

// deno-lint-ignore ban-types no-explicit-any
type Constructor = new (...args: any[]) => {};

function enumProperties<T extends SchemaGenerator>(this: T) {
	const propertyDataList = Reflect.getMetadata(objectProperties, this);
	if (propertyDataList != null) {
		for (const name of Object.keys(propertyDataList)) {
			const propertySchema: JSONSchema7 = {
				description: propertyDataList[name].description,
				enum: propertyDataList[name].enum,
				uniqueItems: propertyDataList[name].uniqueItems,
				default: propertyDataList[name].default,
				examples: propertyDataList[name].examples,
			};
			removeUndefinedValues(propertySchema);

			// I tried to remove this cast and make it use keyof;
			// unfortunately this can't really be done as a subtype could add keys
			const subObj = (this as Record<string, unknown>)[name];
			let items: JSONSchema7 | undefined = undefined;
			if (subObj != undefined && typeof subObj === "object") {
				if ("schema" in subObj) {
					const subSchema = subObj.schema as JSONSchema7;
					if (propertyDataList[name].type == "array") {
						items = subSchema;
					} else {
						const combinedSchema = {
							...subSchema,
							...propertySchema,
						};
						if (propertySchema.description != null) {
							if (subSchema.description != null) {
								combinedSchema.description = propertySchema.description +
									"\n\n" + subSchema.description;
							}
						}
						this.schema.properties![name] = combinedSchema;
						continue;
					}
				}
			}

			propertySchema.type = propertyDataList[name].type ?? err("No type defined for " + name);
			if (items != undefined) {
				propertySchema.items = items;
			}
			this.schema.properties![name] = propertySchema;
		}
		this.schema.required = Object.keys(propertyDataList).filter((name) => propertyDataList[name].required ?? false);
	}
}

export function schema(defSchema?: JSONSchema7) {
	return function <T extends Constructor>(target: T) {
		return class extends target implements SchemaGenerator {
			public readonly schema: JSONSchema7;

			// deno-lint-ignore no-explicit-any
			constructor(...args: any) {
				super(args);
				this.schema = {
					type: "object",
					...defSchema,
					properties: {},
				};
				enumProperties.bind(this)();
			}
		};
	};
}

// Declare x-taplo extension fields
declare module "npm:@types/json-schema" {
	export interface JSONSchema7 {
		"x-taplo"?: {
			hidden?: boolean;
			docs?: {
				main?: string;
				enumValues?: (string | null)[];
				defaultValue?: string;
			};
			links?: {
				key?: string;
				enumValues?: string[];
			};
			initKeys?: string[];
		};
	}
}
