# Triggers
To see an example XML file, see [Example XML](./example-xml)

### All
Returns `true` if every contained trigger returns `true`.
```xml
<All>
    <!-- Triggers -->
</All>
```
### Always
Returns `true`.
```xml
<Always/>
```
### Any
Returns `true` if any contained trigger returns `true`.
```xml
<Any>
    <!-- Triggers -->
</Any>
```
### CurrentNamedArea
Returns `true` if the current named area's name is `name`.
```xml
<CurrentNamedArea><!-- name --></CurrentNamedArea>
```
### InteractionBlockPosition
Returns `true` if the block that was interacted with was at `x`,`y`,`z`.
```xml
<InteractionBlockPosition><!-- x -->,<!-- y -->,<!-- z --></InteractionBlockPosition>
```
### Not
Returns `true` if the contained trigger returns `false`.
```xml
<Not>
    <!-- Trigger -->
</Not>
```
### NotRunningAlready
Returns `true` if the current on action isn't already running.
```xml
<NotRunningAlready/>
```
### PreviousNamedArea
Returns `true` if the previous named area's name is `name`.
```xml
<PreviousNamedArea><!-- name --></PreviousNamedArea>
```
