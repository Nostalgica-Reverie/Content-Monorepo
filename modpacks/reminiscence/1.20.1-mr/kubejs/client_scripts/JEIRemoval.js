//HIDING FROM JEI
JEIEvents.hideItems(event => {

  for (const itemId of itemsToRemove) {

    event.hide(itemId)

  }

})