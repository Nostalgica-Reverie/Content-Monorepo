# UI Definitions

These are `.json` files loaded into the directory and subdirectories of `<namespace>:ui_definitions/`, that can change all the game interfaces.

To make it easier to understand, they will be divided into 3 topics, the `Target UI`, `Apply Conditions` and the `Elements`.

`--> (*explanation) = JSON Entry Note`

## Target UI
It's a way for UI Definition to be applied to interfaces.

### Example
```json
"targetUI": "title_screen",
"targetType": "id" (default)
"targetRange": "instance" (default)
```
### Description
- `targetUI` -> the identifier, class name or screen title [component](https://minecraft.wiki/w/Raw_JSON_text_format), respectively depending on the `targetType`
- `targetType` -> `id` | `class` | `screenTitle`
- `targetRange` -> `instance`(any instance of the selected interface class) | `class` (only the selected interface class)


Identifiers are generally used for vanilla UI classes, since they can basically be named differently depending on the mod loader, but mod UIs don't need them.  
The ones added by default can be found [here](https://github.com/Wilyicaro/FactoryAPI/wiki/Default-UI-Class-Identifiers)

## Apply Conditions
It's a field in the UI Definition or in one of its elements that supports boolean expressions, which controls whether it will be applied or not.  
Once it has not been applied, it can only be applied again when the screen is reloaded (when it is resized, for example).

### Example
```json
"applyCondition": "${windowHeight}>=720"
```
### Description
- `applyCondition` -> accepts boolean values or boolean expressions

## Elements
This is what makes up the UI Definition, elements are used for everything from storing variables to placing widgets on screens.
They are divided by types, each having its function and fields.  
Fields are generally optional, to avoid errors occurring during execution, and those that occur in expressions fails are warned in the console, but do not cause crashes.  

Every element needs to have a name, either as a key if the elements are in a Json Object, or in a field on the element if they are in a Json Array.  
This name is mainly used to store variables, either directly, or as a prefix separated by `.`, in element types like `children`, which have the option of applying its name as a prefix to its subelements.

### Json Object Example
```json
{
  "elements": {
    "tutorialButton": {
      "type": "add_button",
      "x": "(${width}-150)/2",
      "y": 122,
      "width": 150,
      "height": 30,
      "message": {
        "translate": "gui.advancements"
      },
      "spriteOverride": "minecraft:toast/system",
      "highlightedSpriteOverride": "minecraft:toast/tutorial",
      "actions": {
        "open_default_screen": "skin_customization"
      },
      "cool_variable": 10,
      "factoryAPIIsCool": {
        "type": "draw_string",
        "x": "(${width}-${factoryAPIIsCool.component.width})/2",
        "y": {
          "reference": "cool_variable"
        },
        "component": "Factory API Is Cool!"
      }
    }
  }
}
```
### Json Array Example
```json
{
  "elements": [
    {
      "name": "tutorialButton",
      "type": "add_button",
      "x": "(${width}-150)/2",
      "y": 122,
      "width": 150,
      "height": 30,
      "message": {
        "translate": "gui.advancements"
      },
      "spriteOverride": "minecraft:toast/system",
      "highlightedSpriteOverride": "minecraft:toast/tutorial",
      "actions": {
        "open_default_screen": "skin_customization"
      }
    },
    {
      "name": "cool_variable",
      "type": "put_number",
      "value": 10
    },
    {
      "name": "factoryAPIIsCool",
      "type": "draw_string",
      "x": "(${width}-${factoryAPIIsCool.component.width})/2",
      "y": {
        "reference": "cool_variable"
      },
      "component": "Factory API Is Cool!"
    }
  ]
}
```
### Generic Element Description
- `type` -> identifier of the desired element type
- `name` -> name of the element, used if the elements are using a Json Array format

You can use either of these two formats, depending on what you are going to do, one may be more suitable than the other.  
The main advantage of using the Json Array is that the order of the elements is guaranteed regardless of the context, which is often important, especially if there are many elements with renderings that need order.  

### Common Field Types
- **Numeric Expression** -> a string representing a numeric expression that accepts operations, parentheses, functions, and variables

  **Operators**
  - `+` -> addition
  - `+` -> subtraction
  - `*` -> multiplication
  - `/` -> division
  - `%` -> modulus
  - `&` -> and
  - `|` -> or
  - `^` -> exclusive-or
  - `>>` -> arithmetic shift right operation
  - `<<` -> arithmetic shift left operation
  
  **Functions**
  - `sqrt` -> square root (`sqrt x`)
  - `cbrt` -> cube root (`cbrt x`)
  - `pow` -> power (`pow x y`)
  - `min` -> the smaller number of the two (`min x y`)
  - `max` -> the bigger number of the two  (`max x y`)
  - `clamp` -> the first number limited to the range of the two sequential ones (`clamp x y z`)

- **Boolean Expression** -> a string representing a boolean expression that accepts equalities, logical operations and variables

  **Equalities**
  - `==` -> equal
  - `!=` -> different
  - `>` -> bigger
  - `<` -> smaller
  - `>=` -> bigger or equal 
  - `<=` -> smaller or equal

  **Logical Operators**
  - `&&` -> and
  - `||` -> or
  - `!` -> not
  - `&` -> bitwise and
  - `|` -> bitwise or

Both types of expressions have the same variable format, which is `${<variable_name>}`, where `<variable_name>` is the name of the desired variable.

Also in both types it is possible to use a fallback value, which has the following format: `${leftPos}:0`
This prevents the expression from failing, and obviously provides the fallback value instead.
