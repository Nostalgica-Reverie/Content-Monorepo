# Effects
To see an example XML file, see [Example XML](./example-xml)

### AppendToDialog
Appends to an existing dialog `Text`.
```xml
<AppendToDialog><!-- Text --></AppendToDialog>
```
Closes a dialog, clearing all text and all items.
### CloseDialog
```xml
<CloseDialog/>
```
Shows hardcoded string "Press {y} to continue.".
### ContinueText
```xml
<ContinueText/>
```
Pauses execution of effects for 40 ticks.
### Delay
```xml
<Delay/>
```
Shows a new dialog with text `Text`, clearing the previous text and items.
### Dialog
```xml
<Dialog><!-- Text --></Dialog>
```
### DialogSequence
Asynchronously runs `Effects`.
```xml
<DialogSequence>
    <!-- Effects -->
</DialogSequence>
```
### ExitDialogSequence
Stops running of effects immediately.
```xml
<ExitDialogSequence/>
```
### HardcodedStatusMessage
Shows message `Entered named area {current named area}` in the status bar.
```xml
<HardcodedStatusMessage/>
```
### Log
Don't use this
```xml
<Log><!-- Text --></Log>
```
### PopulateAllContainersInsideOfNamedArea
Populates all containers inside of a player's current named area based on the dimension's game rules file.
```xml
<PopulateAllContainersInsideOfNamedArea/>
```
### ProceedCancel
Pauses the execution of effects until it receives user input on whether to proceed or cancel (currently "y" or "n" from chat).
```xml
<ProceedCancel>
    <Proceed>
        <!-- Effects -->
    </Proceed>
    <Cancel>
        <!-- Effects -->
    </Cancel>
</ProceedCancel>
```
### ShowItemInDialog
Shows `Item` in the current dialog. `Item` is a Minecraft item, specified with the same format used in commands.
```xml
<ShowItemInDialog><!-- Item --></ShowItemInDialog>
```
### StopAll
Stops execution of all effects of all on actions.
```xml
<StopAll/>
```
### WaitForOnAction
Pauses the execution of effects until the following on action has its conditions met.
```xml
<WaitForOnAction>
    <!-- On Action -->
</WaitForOnAction>
```
