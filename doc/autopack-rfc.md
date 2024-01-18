- Feature Name: auto-pack
- Start Date: 2022-04-21

# Summary

A tool enabling web developers to use their favorite boilerplate-creating zero-config framework like CRA and but run the app in a setup similar as prod without - sacrificing the charm of the expected local development workflow or/and expecting them to get a DevOps certification. A tool to end the issue of *it runs fine in local but fails in the deployment pipeline*.

# Motivation

Modern enterprise web developers use boilerplate-creating tools like CRA that enhances their productivity when it comes to local development and also to produce a production-ready build artifact(s). However, the same developer is on their own when it comes to taking the built artifact(s) and packaging it in a way that it runs everywhere.

There is a gap both from technical knowledge as well tooling perspective when it comes for the web developers to have the necessary softwares installed and configurations set to make their application deployable in foreign environments with the same level of sophistication / automation  that they get for their local development.

Most of the time web developers delegate these tasks to a different team (DevOps). And only would realize the issues with their package if/when some form of a continuous deployment pipeline fails to run the same package. This often becomes a contentious issue between developers and DevOps that impacts normal release workflow leading to impacting the overall project delivery.

Containers today solves the issues of the past when it comes to have reproducible software deployment across various host platforms. However, to create container images from application's source code require decent amount of domain knowledge of unix shells, unix networking / disk I/O, and some general virtualization concepts. Containerizing web applications has its own challenges thanks to the complexity of writing a modern web application itself. So someone has to always create the container images post-facto looking/understanding the semantics of overall application design.

What if we automate this whole process where developers don't have to change much in their local development workflow and not only get a production-ready build artifact but also a distributable artifact (container image) which takes care of cross-platform deployment issues.

# Guide-level explanation

As a web developer today working with modern front-end frameworks like React I have a standard development workflow -
- I use my favorite boilerplate-creating tool like CRA to build a fresh application scaffolding for my new awesome project
- open the project folder in my favorite IDE like VSCode
- run the local development server using `npm run start`,
- start making changes to the source code.

In the above workflow one of the important gain that I get is the instant feedback in the form of auto-reloading browser. I as a web developer only concern myself to write application specific code and the boilerplate-creating tool takes care of the rest to finally show it in the browser either some error in my code or the actual web application.

I want the tool here to behave in such a manner. I want to just keep on writing the same application code and I expect the tool to create the relevant deployment artifact. Better use the deployment artifact to deploy in some locally installed runtime. And much cooler would be if the application that today runs locally in the browser from some adhoc web server located within the node_modules folder (this is something [Webpack Dev server](https://webpack.js.org/configuration/dev-server/) provides) would actually gets served from the runtime after loading the deployment artifact !

## Pre-Requisite

Developers today need to have Node and VSCode installed for them to start creating any web application. For the time being, they would need one more software for this tool to work - Docker.

## Installation

This tool will be a cross-platform CLI executable. Usually it means that for this tool to work it need to be in the system path. For the rest of discussion we will use the name of the tool to be `auto-pack`

## Usage

`auto-pack`  is a command-line utility. It generally performs tasks in the background but also have some minimum set of commands

```bash
$ auto-pack init
```

`init` command initializes the project by doing some cursory checks to see if the relevant software is already installed. Right now it will check for Docker's availability in the system and whether it's already running. It will also take necessary steps to start any pre-requisites if not started already.

Along with that it will also take care to install and configure any external dependencies required. Right now it might need to install Buildpack's CLI.

Once the dependencies are installed and configured, this command will try to prime the project thereby creating and caching some default layers that might future processing faster.

`init` might also change relevant `npm` scripts in project's `package.json` so that `auto-pack` could channel the usual project build and run commands.

Finally, `init` would ensure that `auto-pack` runs as a daemon in the background.

```bash
$ auto-pack run
```

`run` command would try to launch the packed image as a container and also do the necessary steps to wire the host system hardware ports & launch the relevant browser to render the app. Its job is to kind of mimic the similar experience that `npm run start` provides for a CRA built app.

```bash
$ auto-pack export
```

`export` command would try to export the image & other relevant artifacts either directly to some registry or to some distributable format. Right now it might try to publish the image to some docker registry

```bash
$ auto-pack show
```

`show` command would try to provide information about the images and running containers (if any). It would also provide information about the tool itself.

```bash
$ auto-pack stop
```

`stop` command would try to stop the background process

```bash
$ auto-pack clean
```

`clean` command would clean up any stalled processes, intermediate logs, and other temporary files. It will also try to clean up images & running containers.

## Expected workflow

The intent of the tool is not to change developer's current workflow drastically. Once the developer has setup their project using a boilerplate-creating tool they install `auto-pack` executable and run `auto-pack init`. From that point - `auto-pack` should run in the background and developers should have no change in the way they interact with their application during their usual development process.

Developers should have option to run their application using the standard `npm` scripts provided as part of their boilerplate creating tool or use the new scripts that will leverage `auto-pack` to run the same application but this time by serving the distributable artifact from within the container which `auto-pack` should have generated & launched while it was running in the background.

## CI/CD Pipeline Usage

# Reference-level explanation

The idea of `auto-pack` is to leverage the concept of creating container images without a Dockerfile. Tools like [Buildpack](https://buildpacks.io/) & [source-to-image (s2i)](https://github.com/openshift/source-to-image) takes this concept to the point where they could automatically generate container images from source code. Such tools work as per this sequence of tasks

```
            Detect --> Build --> Export
```

- **Detect**: detects from source code to pick the right base image
- **Build**: install dependencies and run the build command
- **Export**: create OCI image

`auto-pack` apart from the above tasks will also be doing

- **SPA Server**: `auto-pack` will create an automatic Node server to serve the SPA which will provide few functionality by default like dynamic configuration management & efficient static file caching. In case a project already has some server `auto-pack` will have a way to leverage the same in place of it's own server.
- **Launch**: once exported the image will be used to launch within a container runtime and also render the web application by (re)launching a relevant browser
- **Watch**: watch for file changes &initiate Build, Export, and Launch

```
  Detect --> SPA Server --> Build --> Export --> Launch
                              ^                     |
                              |                     v
                              +----------Watch------+
```

## Extendable Design

`auto-pack`  will use Buildpack internally to create OCI images from source code. However, the implementation of the same will be made following a [Bridge](https://en.wikipedia.org/wiki/Bridge_pattern) design pattern so that the relevant tasks are abstracted from the actual implementation using some tool. That tool might be Buildpack right now but that also could change in future.

## Rust as the implementation language

Implementing a tool like `auto-pack` would require system level tasks such as running processes in background mostly as a daemon, doing efficient disk I/O, working with OS level concurrency when it comes to file watching, and finally working efficiently as a CLI across platforms. It is necessary for implementation of `auto-pack` to happen in a system language and not in a high-level programming language so that we don't have to deal with system-level optimizations later. Moreover, we also want the implementation of `auto-pack` to be following a typed functional programming design so that we have solid guarantees from the type system during construction and have other efficiencies that a functional programming thinking provides.

Rust is the only systems-programing language that matches our desired criteria. It has a sound and highly sophisticated type system that will not only guide us during implementation but also keep our implementation safe. It's system-level support via language primitives & supporting libraries will help having an efficient `auto-pack`' implementation. Moreover, today [Rust is considered to be the future of JS infrastructure](https://leerob.io/blog/rust) given the kind of adoption Rust is having across JS community.


## Custom Buildpack

`auto-pack` will be leveraging Buildpack to create OCI images without Dockerfiles. Buildpack today provides a nice design via which source code of any language are converted into OCI images. There are already efficient base build packs available for JS projects especially from [Paketo](https://paketo.io/) which `auto-pack` will leverage.

However, for `auto-pack` we will create a custom buildpack which will have its own detection & building routine. The initial design will focus on some common patterns that today's boilerplate-creating frameworks like CRA, Vite are adopting when they are building the dev server for local development. `auto-pack` will hook into those places to have an efficient build routine.

Moreover, `auto-pack` will be having its own custom process of layer caching based on our understanding of different types of files that are generated for a modern web application. Every source code change won't generate all new files especially the static assets and hence such could be well cached. A custom buildpack would contain such instructions & more.

## Background Process

`auto-pack` will be working as a directory specific daemon. Core reason for this design decision is because of the nature of tasks that `auto-pack` will do. The tasks of generating images from source code might take a long time (initially) which will surely impact developer productivity. We don't want developers to change their normal workflow but still get the advantage of a local containerized deployment and execution of their web application. And hence doing the resource consuming tasks in the background without impacting developer's main workflow will be beneficial.

`auto-pack` will be doing following tasks in the background
- **Build**: generating the production build from the current source code of the project
- **Creating Image**: using the build to create the OCI image
- **Creating Container**: using the image to create a container in the available runtime (for now it would be docker)
- **File Watching**: watching for file changes to do incremental building and re-creating images & containers

## Incremental builds

`auto-pack` will be optimizing the build routine on top of the build artifacts produced by the boilerplate-creating tools like CRA. it is going to help `auto-pack` provide a faster image creation & execution feedback. The build produced by most boilerplate-creating tools are in the form of JS bundles (this might change in future once native ES modules are widely used for production build). `auto-pack` will try to use a content-hashing based approach to identify and replace built artifacts in running containers leveraging Buildpack's layer rebasing strategies.

# Drawbacks

Generating images without Dockerfile is not a mainstream approach. And when things happen in the background it becomes difficult to diagnose any issues. `auto-pack` will try to smoothen all this with efficient logging, helpful info messages, and proper CLI command options.

Tools that try to leverage OS processes to do background file monitoring  and disk I/O at the same time might suffer from standard issues like stalled processes, zombie daemons, and resource draining threads. `auto-task` will be leveraging Rust's powerful [Tokio](https://tokio.rs/) suite of APIs to safely manage concurrent I/O processes and will use a Supervision Trees based approach to monitor processes.

Generating & executing images without Dockerfile would surely help developers achieve productivity when it comes to running & testing their application in a production environment locally. However, it might be an issue for DevOps teams who are generally tasked to containerize the application via CI/CD pipelines. And there Dockerfiles are still the preferred way to create images. Even though there are [CI/CD platforms](https://buildpacks.io/docs/tools/) that support Cloud Native Buildpacks (CNB) to create images from source code but in enterprises that process is not as prevalent as it should be. `auto-pack` being a cross platform CLI executable would ensure that it works in CI/CD platforms (which are mostly linux based). Once the CI/CD pipeline environment has docker engine available then `auto-pack`  would gather all its relevant dependencies during initialization hence for DevOps it would as good as replacing some variation of `docker build` command with `auto-pack init && auto-pack export` - these series of command would first initialize and then do the necessary steps to export an image out of the source code.

# Rationale and alternatives

There are tools today which either provide some way to build CNI images without the requirement of Docker to be available as a deamon or Dockerfile to be available. `auto-pack` is mostly concerned with the later use case. 

Cloud Native Buildpacks are the most preferred way to generate images from source code today. However, it comes with its own learning curve which might not be as daunting as Docker itself but still it requires a standard web developer proficient in React to learn decent amount of lingo from world of containers. And similarly there are other tools to generate images directly without Dockerfile like [creating Docker images with Nix package manager](https://nix.dev/tutorials/building-and-running-docker-images) and [Jib](https://github.com/GoogleContainerTools/jib). Jib comes close to something what CNBs are doing - i.e. directly producing CNI images from source code. However, where CNB is a generic specification that can be operationalized with applications built in any language / platform, Jib is only specific to Java based applications. Creating docker images with Nix is as generic as writing Dockerfiles but in a more sophisticated & expressive programming language (unlike the Dockerfile syntax which is an adhoc configuration language syntax lacking expressiveness and usual developer experience). However, it comes with additional requirements of a presence of a system level tool (i.e. Nix itself) and/or managing tool specific configurations along with application source code.

Intent of `auto-pack` is to make it easier for web developers (the target users) to venture the world of containers and hence it does not have to bother about requirements of other set of users. That constraint also enables `auto-pack` to have specific customizations that will be relevant and beneficial for modern web developers using tools like CRA. And again it can be safely used as a local only tool while CI/CD platforms could use known tools like Dockerfiles.

Apropos the above current state of tools to build CNI images without Dockerfiles and the unique mission of `auto-pack`, Cloud Native Buildpacks based backend to create CNI images has been considered to be the choice of underlying technology to create CNI images directly from code. Since the focus is to balance between providing similar developer experience that modern web-developers expect from CLI tools and also to not re-inventing the wheel of creating something from scratch, `auto-pack` is an attempt to create a simple automaton over Cloud Native Buildpacks for a very specific set of users i.e. modern web developers using React specific tools like CRA to develop their applications.
# Prior art

`auto-pack` will leverage ideas from various tools. The basic idea of generating container images from source code was first seen in Nix and then popularized as CNB. The idea of running processes in the background to improve the efficiency of the activities happening in the foreground is something that Unix daemons & Kernel level processes generally do. There are many designs captured in [The Architecture of Open Source Applications](http://aosabook.org/en/index.html) which kind of elaborates how to do the same efficiently for a reduced scope tool like `auto-pack`. Idea of doing incremental builds via content hashing is something that build tools like [Bazel](https://bazel.build/docs/build#correct-incremental-rebuilds does pretty efficiently. `auto-pack` will leverage some patterns from those implementations.

# Unresolved questions

- How can `auto-pack` hook its logs into the UX of `npm run start` provided by CRA ? Or will it be fine if it works via a different npm script like `npm run start:pack` ? The later would reduce its usage though.
- Should `auto-pack` start the background process as soon as users goes inside project's directory like how tools like [direnv](https://direnv.net/) works ? Or it should only start on users action ? We can also have a configuration to decide.
- How to manage the proliferation of docker images which would get created for every change the developer would make ?

# Future possibilities

`auto-pack` will start with limited scope to support specific types of projects (React projects built using CRA) with some fixed conventions. Reason for the same is to have the initial version properly tested. However, future versions should have ways to support more type of projects and should have a way to support configurations.

The requirement of the presence of Docker as engine / daemon with root level privileges for `auto-pack` to work would change once native OCI builders & runtimes like [Buildah](https://github.com/containers/buildah/blob/main/README.md) and [Podman](https://github.com/containers/podman) respectively are widely available across platforms especially in Windows. For `auto-pack` to work in Windows which is the dominant platform of web developers in enterprises Docker is an unfortunate dependency. The future of containerized distribution of applications should be to create OCI images directly from code and running it in some rootless OCI runtime.

Although `auto-pack` would always be a CLI tool it would be beneficial if we could also have it as an editor plugin. That way we can provide much better UX for developers. Modern IDEs like VSCode provide tools like [Language Server Protocol](https://microsoft.github.io/language-server-protocol/) which can also eliminate `auto-pack` to run as daemon and rather have a nice interface similar to how other Language Server Protocol servers work.
