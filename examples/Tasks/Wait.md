# Wait Task

This task will make the automation to wait for X amount of millisecondes before continuing 

## Fields (Required)
* Name: A small decription of what the taks will do.
* duration_ms: Time in millisecondes

### Optional
* element: Locating the elements based on the provided locator values
    #### Locator strategies:
    * id
    * xPath
    * className


## Example

```
  - name: "wait 5 sec"
    wait:
      duration_ms: 5000
```      
## Wait for element to be displayed
* xPath
  ```
    - name: "wait for element"
      wait: 
        element:
          xPath: //*[@id="search-form"]/fieldset/button
  ```      
* id
  ```
    - name: "wait for element"
      wait: 
        element:
          id: "search-form"
  ```
* className
  ```
    - name: "wait for element"
      wait: 
        element:
          className: 'pure-button-primary-progressive'
  ```        