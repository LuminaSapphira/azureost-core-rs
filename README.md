# azure-ost-core
Work-in-progress core for new module system of azureost-rs.
This will be a refactored and cleaned up version of my current azureost-rs repository,
without the command-line interface (so just a library at that point). The CLI will be
moved into its own crate, azureost-cli, and another crate, azureost-jni, will be made
to provide functionality for the Java Native Interface, allowing a GUI written in Java
to be created.

The new system will function as follows:
- **azureost-core-rs** - Core functionality for azureost-rs
  - **azureost-cli** - Command-line interface for the application
  - **azureost-jni** - Java Native Interface library for the application
    - **azureost-jgui** - GUI written in Java (possibly) that executes calls to the JNI (then to the core)
