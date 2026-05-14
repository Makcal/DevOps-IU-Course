{ pkgs ? import <nixpkgs> {} }:
let
  lib = pkgs.lib;
  srcClean = lib.cleanSourceWith {
    src = ./.;
    filter = path: type:
      let
        relPath = lib.removePrefix (toString ./. + "/") (toString path);
      in 
        (  lib.hasPrefix "src" relPath
        || relPath == "main.py"
        || relPath == "setup.py");
  };
in with pkgs.python3Packages; buildPythonApplication {
  pname = "devops-info-service";
  version = "1.0";
  format = "setuptools";

  src = srcClean;
  propagatedBuildInputs = [
    fastapi
    uvicorn
    prometheus-client
  ];
}
