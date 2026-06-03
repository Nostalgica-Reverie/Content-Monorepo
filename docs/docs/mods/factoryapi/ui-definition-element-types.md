# UI Definition Element Types
The following UI Definition Element Types are added by default to the Factory API, but other mods that integrate with it may add others

- `children` -> **Organizes the elements within the JSON file and allows you to add a prefix to their name**
   - `applyPrefix` -> true by default, accepts only boolean values ​​and controls whether the prefix, which is the name of this element, will be applied to its subelements
   - `elements` -> UI Definition elements

## Widget Element Types
The following Element Types are related to widgets, and share the following fields:
  - `message` -> a component defining this widget message
  - `x` -> x position of this widget, it's 0 by default (integer or integer expression)
  - `y` -> y position of this widget, it's 0 by default (integer or integer expression)
  - `width` -> width of this widget (integer or integer expression)
  - `height` -> height this widget (integer or integer expression)
  - `order` -> order in which this widget will be added, its default is `${renderablesCount}`(integer or integer expression)
  - `isVisible` -> controls the visibility of this widget, without removing it (boolean or boolean expression)

***

- `add_button` -> **Adds a button to the interface**
   - `actions` -> an array of actions, that will be executed when pressing this button

- `add_checkbox` -> **Adds a checkbox to the interface**
   - `selected` -> accepts boolean values or boolean expressions, defining the initial state of this checkbox
   - `actions` -> an array of actions, that will be executed when pressing this checkbox

- `add_slider` -> **Adds a slider to the interface**
   - `selected` -> accepts boolean values or boolean expressions, defining the initial state of this checkbox
   - `entries` -> an array of cycle actions entries, that will be executed when changing the selected cycle entry

- `modify_widget` -> **Modify widgets on the interface**
   - `range` -> an integer range defining the indexes of the widgets to be modified
   - `index` -> an integer defining the widget to be modified, if a range isn't provided
   - `actions` -> an array of actions, that will override the actions in this widget

***

- `remove_widget` -> **Remove widgets on the interface**
   - `range` -> an integer range defining the indexes of the widgets to be removed
   - `index` -> an integer defining the widget to be removed, if a range isn't provided

- `put_number` -> **Stores a number variable**
   - `value` -> accepts number values or number expressions

- `put_component` -> **Stores a component variable and its width**
   - `value` -> a component

- `put_string` -> **Stores a string variable**
   - `value` -> a string

- `put_boolean` -> **Stores a boolean variable**
   - `value` -> accepts boolean values or boolean expressions

- `put_resource_location` -> **Stores a resource location**
   - `value` -> a resource location

- `put_vec3` -> **Stores a vec3**
   - `value` -> a vector with 3 axes

## Render Element Types
The following Element Types are related to rendering, and share the following fields:
  - `translateX` -> x translation of this render, it's 0 by default (float or float expression)
  - `translateY` -> y translation of this render, it's 0 by default (float or float expression)
  - `translateZ` -> z translation of this render, it's 0 by default (float or float expression)
  - `scaleX` -> x scale of this render, it's 0 by default (float or float expression)
  - `scaleY` -> y scale of this render, it's 0 by default (float or float expression)
  - `scaleZ` -> z scale of this render, it's 0 by default (float or float expression)
  - `renderColor` -> color of this render, it's #FFFFFFFF (white) by default (integer or integer expression)
  - `amount` -> amount of times this will be rendered (integer or integer expression)
  - `order` -> order in which this render will be added, its default is `${renderablesCount}` (integer or integer expression)
***

- `blit` -> **Renders a texture on the interface**
   - `texture` -> a resource location defining the texture to be rendered
   - `x` -> x position to start the render, it's 0 by default (integer or integer expression)
   - `y` -> y position to start the render, it's 0 by default (integer or integer expression)
   - `uvX` -> x position in the texture, it's 0 by default (integer or integer expression)
   - `uvY` -> y position in the texture, it's 0 by default (integer or integer expression)
   - `width` -> width of this render, it's 0 by default (integer or integer expression)
   - `height` -> height of this render, it's 0 by default (integer or integer expression)
   - `imageWidth` -> width of the texture of this render, it's 256 by default (integer or integer expression)
   - `imageHeight` -> height of the texture of this render, it's 256 by default (integer or integer expression)

- `blit_sprite` -> **Renders a gui sprite on the interface**
   - `texture` -> a resource location defining the sprite to be rendered
   - `x` -> x position to start the render, it's 0 by default (integer or integer expression)
   - `y` -> y position to start the render, it's 0 by default (integer or integer expression)
   - `width` -> width of this render, it's 0 by default (integer or integer expression)
   - `height` -> height of this render, it's 0 by default (integer or integer expression)

- `fill` -> **Renders a color on the interface**
   - `texture` -> a resource location defining the sprite to be rendered
   - `x` -> x position to start the render, it's 0 by default (integer or integer expression)
   - `y` -> y position to start the render, it's 0 by default (integer or integer expression)
   - `width` -> width of this render, it's 0 by default (integer or integer expression)
   - `height` -> height of this render, it's 0 by default (integer or integer expression)
   - `color` -> color of this render, it's #FFFFFFFF (white) by default (integer or integer expression)

- `fill_gradient` -> **Renders a color gradient on the interface**
   - `texture` -> a resource location defining the sprite to be rendered
   - `x` -> x position to start the render, it's 0 by default (integer or integer expression)
   - `y` -> y position to start the render, it's 0 by default (integer or integer expression)
   - `width` -> width of this render, it's 0 by default (integer or integer expression)
   - `height` -> height of this render, it's 0 by default (integer or integer expression)
   - `color` -> color of this render, it's #FFFFFFFF (white) by default (integer or integer expression)
   - `secondColor` -> second color of this render, it's #FFFFFFFF (white) by default (integer or integer expression)

- `draw_string` -> **Renders a text on the interface**
   - `component` -> a component defining this render text
   - `x` -> x position to start the render, it's 0 by default (integer or integer expression)
   - `y` -> y position to start the render, it's 0 by default (integer or integer expression)
   - `shadow` -> defines whether the text will have a shadow, it's true by default (boolean or boolean expression)
   - `color` -> color of this render, it's #FFFFFFFF (white) by default (integer or integer expression)

- `render_item` -> **Renders a item on the interface**
   - `item` -> item to be rendered, it's empty by default
   - `x` -> x position to start the render, it's 0 by default (integer or integer expression)
   - `y` -> y position to start the render, it's 0 by default (integer or integer expression)
   - `isFake` -> defines whether the item will be rendered as fake, it's false by default (boolean or boolean expression)
   - `allowDecorations` -> defines whether the item will be rendered with its decorations, it's true by default (boolean or boolean expression)

- `render_items` -> **Renders items on the interface, having `<element_name>.index` available to be used in the rendering**
   - `items` -> item array to be rendered, it's empty by default
   - `x` -> x position to start the render, it's 0 by default (integer or integer expression)
   - `y` -> y position to start the render, it's 0 by default (integer or integer expression)
   - `isFake` -> defines whether the item will be rendered as fake, it's false by default (boolean or boolean expression)
   - `allowDecorations` -> defines whether the item will be rendered with its decorations, it's true by default (boolean or boolean expression)

***

- `draw_multiline_string` -> **Adds a multiline string layout element to the interface**
   - `component` -> a component defining this render text
   - `x` -> x position to start the render, it's 0 by default (integer or integer expression)
   - `y` -> y position to start the render, it's 0 by default (integer or integer expression)
   - `width` -> width of the multiline string, it's 0 by default (integer or integer expression)
   - `lineSpacing` -> line spacing of the multiline string, it's 12 by default (integer or integer expression)
   - `shadow` -> defines whether the text will have a shadow, it's true by default (boolean or boolean expression)
   - `color` -> color of this render, it's #FFFFFFFF (white) by default (integer or integer expression)
   - `centered` -> defines whether the text will be centered, it's false by default (boolean or boolean expression)

- `compare_items` -> **Stores a boolean variable comparing two items**
   - `firstItem` -> first item to be compared, it's empty by default
   - `secondItem` -> second item to be compared, it's empty by default
   - `checkCount` -> defines whether this comparison will check the count of the items, it's true by default (boolean or boolean expression)

- `chance` -> **Stores a random integer variable based on a minimum and a maximum value**
   - `min` -> minimum random value, it's 0 by default (integer or integer expression)
   - `max` -> maximum random value, it's 0 by default (integer or integer expression)
