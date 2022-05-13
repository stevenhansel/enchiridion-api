package main

import (
	"flag"
	"fmt"
	"os"

	"github.com/stevenhansel/enchridion-api/internal/config"
	"github.com/stevenhansel/enchridion-api/internal/container"
	internalHttp "github.com/stevenhansel/enchridion-api/internal/http"
)

func main() {
	var env config.Environment

	flag.Var(
		&env,
		"env",
		"application environment, could be either (development|production)",
	)

	flag.Parse()
	
	container, err := container.New(env)
	if err != nil {
		fmt.Fprintf(os.Stderr, err.Error())
		os.Exit(1)
	}

	http := internalHttp.New(container)
	http.Serve()
}
