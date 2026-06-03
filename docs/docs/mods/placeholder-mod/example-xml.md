# Example XML
Feel free to copy this into a text editor for reference.

```xml
<!-- Mace tutorial -->
<EnteredNamedArea>
    <Trigger>
        <All>
            <Any>
                <All>
                    <CurrentNamedArea>maceArea</CurrentNamedArea>
                    <Not><PreviousNamedArea>maceTopElevatorArea</PreviousNamedArea></Not>
                </All>
                <All>
                    <CurrentNamedArea>maceTopElevatorArea</CurrentNamedArea>
                    <Not><PreviousNamedArea>maceArea</PreviousNamedArea></Not>
                </All>
            </Any>
            <Not><PreviousNamedArea>maceBottomArea</PreviousNamedArea></Not>
            <NotRunningAlready/>
        </All>
    </Trigger>
    <Effects>
        <DialogSequence>
            <Dialog>
                There are some Maces in this area's chest.
            </Dialog>
            <ShowItemInDialog>minecraft:mace</ShowItemInDialog>
            <Delay/>
            <AppendToDialog>
                Press {y} to learn more about Maces.
                Press {n} if you already know about Maces.
            </AppendToDialog>
            <ProceedCancel>
                <Cancel>
                    <ExitDialogSequence/>
                </Cancel>
            </ProceedCancel>
            <Dialog>Maces are weapons that use downward momentum to slam your opponent. Maces can receive a variety of useful enchantments to power you up.</Dialog>
            <Delay/>
            <ContinueText/>
            <ProceedCancel/>
            <Dialog>Maces get more effective the higher you fall from. Landing a hit deals massive damage and cancels any fall damage you would have taken, but missing a hit can be catastrophic.</Dialog>
            <Delay/>
            <ContinueText/>
            <ProceedCancel/>
            <Dialog>You can craft a Mace by placing a Heavy Core on top of a Breeze Rod.</Dialog>
            <Delay/>
            <ContinueText/>
            <ProceedCancel/>
            <Dialog>In front of you is a large hole, you can spawn an armor stand to use as target practice.</Dialog>
            <Delay/>
            <AppendToDialog>Press the stone button to continue.</AppendToDialog>
            <WaitForOnAction>
                <BlockActivated>
                    <Trigger>
                        <Any>
                            <InteractionBlockPosition>-61,76,17</InteractionBlockPosition>
                            <InteractionBlockPosition>-61,76,27</InteractionBlockPosition>
                            <InteractionBlockPosition>-59,76,17</InteractionBlockPosition>
                            <InteractionBlockPosition>-59,76,27</InteractionBlockPosition>
                        </Any>
                    </Trigger>
                </BlockActivated>
            </WaitForOnAction>
            <Dialog>
                Jump down the hole and swing your mace at the armor stand just before you hit the ground. There is a pair of feather falling boots in the chest, take one to help break your fall if you miss.
            </Dialog>
            <WaitForOnAction>
                <EnteredNamedArea>
                    <Trigger>
                        <CurrentNamedArea>maceBottomArea</CurrentNamedArea>
                    </Trigger>
                </EnteredNamedArea>
            </WaitForOnAction>
            <Dialog>
                At the back of this room is a wind charge launcher that will send you back to the top of this hole. Stand inside the composter.
            </Dialog>
            <Delay/>
            <AppendToDialog>Press the stone button to continue.</AppendToDialog>
            <WaitForOnAction>
                <EnteredNamedArea>
                    <Trigger>
                        <CurrentNamedArea>maceTopElevatorArea</CurrentNamedArea>
                    </Trigger>
                </EnteredNamedArea>
            </WaitForOnAction>
            <Dialog>
                Inside the chest there are 3 Mace enchantments. The first one is Breach, which will make Maces more effective against armored foes.
            </Dialog>
            <Delay/>
            <ContinueText/>
            <ProceedCancel/>
            <Dialog>
                Wind Burst will cause the enemy to explode in a powerful burst of wind, launching you back up and letting you chain hits to deal massive damage in quick succession.
            </Dialog>
            <Delay/>
            <ContinueText/>
            <ProceedCancel/>
            <Dialog>
                Lastly there's Density, which will multiply the damage you do when you fall. Pairing this with wind charges or an elytra can let you clear out enemies in mere moments!
            </Dialog>
            <Delay/>
            <ContinueText/>
            <ProceedCancel/>
            <CloseDialog/>
        </DialogSequence>
    </Effects>
</EnteredNamedArea>
```
