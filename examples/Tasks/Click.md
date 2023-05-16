# Click Task

This task combines moving to the center of an element with pressing and releasing the left mouse button. This is otherwise known as “clicking”

## Fields (Required)
* Name: A small decription of what the taks will do.
* element: Locating the elements based on the provided locator values
    #### Locator strategies:
    * id
    * xPath
    * className

## Example
* xPath

    ```
    - name: "Click search button"
      click:
        element:
            xPath: '//*[@id="search-form"]/fieldset/button'
    ```      
* id
    ```
    - name: "Click search button"
      click:
        element:
            id: "search-form"
    ```   
* className   
    ```
    - name: "Click search button"
      click:
        element:
            className: 'pure-button-primary-progressive'
    ```   
## Variables support
```
  - name: "Click search button"
    click:
      element:
        xPath: "{click}"
```        