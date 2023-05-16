# Set Variables Task

This task allows you to add variables to be used inside other tasks

## Fields 
### Required
* Name: A small decription of what the taks will do.
* set_vars: Variables you want to use in other tasks -> `name: value`

### Supported Tasks
* send_key
* screenshot
* click

## Example
```
  - name: "set vars"
    set_vars:
      input: "Rust"
      enterKey: "\uE007" 
      xPath: '//*[@id="searchInput"]'
```      
