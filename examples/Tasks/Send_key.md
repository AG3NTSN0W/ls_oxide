# Send Key Task

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
    - name: "enter rust in search"
      send_key:
        input: "Rust"
        element:
            xPath: '//*[@id="searchInput"]'
    ```      
* id
    ```
    - name: "enter rust in search"
      send_key:
        input: "Rust"
        element:
            id: "searchInput"
    ```      
* className   
    ```
    - name: "enter rust in search"
      send_key:
        input: "Rust"
        element:
            className: '//*[@id="searchInput"]'
    ```       
## Variables support
```
    - name: "enter rust in search"
      send_key:
        input: "{input}"
        element:
            xPath: "{xPath}"
```    