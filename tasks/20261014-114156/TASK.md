# Move components to the bottom

- STATUS: OPEN
- PRIORITY: 120
- TAGS: v0.1.5

Now components block are have to be inside of endpoint description, but better approac is to make it 
mutual on single files, like components itself can also be mutual so dot link them to certain endpoint logic.
But better would be if they would call "General" or "Common" components ("Общие" in russian); and components inside of 
endpoint better would be called an "Internal" components. We have to allow "General" components be mandatory used somewhere,
i.e. something have to be linked to such component - error otherwise. For endpoint components rule work backwards - it cannot be used
anywhere except of it endpoint. If such thing found suggest to move to general components OR stop referring to it - also error.
