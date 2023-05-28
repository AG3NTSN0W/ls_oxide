# Screenshot Task

This task will take a screenshot of the current window or of an element.

## Fields 
### Required
* Name: A small decription of what the taks will do.
* Path: The location where the screenshot needs to be saved.

### Optional
* element: Locating the elements based on the provided locator values
    #### Locator strategies:
    * id
    * xPath
    * className

## Example
* window
    ```
    - name: "Take a screenshot"
      screenshot: 
        path: "/tmp/screenshot.png"
    ```    

* xPath

    ```
    - name: "Take a screenshot"
      screenshot: 
        path: "/tmp/screenshot.png"
        element:
            xPath: '//*[@id="mw-content-text"]/div[1]/div[4]'
    ```      
* id
    ```
    - name: "Take a screenshot"
      screenshot: 
        path: "/tmp/screenshot.png"
        element:
            id: "mw-content-text"
    ```      
* className   
    ```
    - name: "Take a screenshot"
      screenshot: 
        path: "/tmp/screenshot.png"
        element:
            className: 'vector-toc-level-1-active vector-toc-list-item-active'
    ```     
## Variables support
```
    - name: "Take a screenshot"
      screenshot: 
        path: "{path}"
        element:
            xPath: "{xPath}"
```    
