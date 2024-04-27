package main

import "fmt"
import "C"

//export echo
func echo() {
	fmt.Println("echo")
}

func main() {}
