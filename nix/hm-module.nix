self:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.downer;
  desktop = "io.github.DisposableSoftware.Downer.desktop";
  ini = pkgs.formats.ini { };
in
{
  options.programs.downer = {
    enable = lib.mkEnableOption "Downer, a minimal GTK4 Markdown viewer";
    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${pkgs.stdenv.hostPlatform.system}.default;
      defaultText = lib.literalExpression "downer.packages.\${system}.default";
      description = "The Downer package to install.";
    };
    settings = lib.mkOption {
      inherit (ini) type;
      default = { };
      example = {
        appearance = {
          flavor = "latte";
          accent = "blue";
          opacity = 0.9;
        };
      };
      description = "Written to $XDG_CONFIG_HOME/downer/config.ini. See config.example.ini in the package's share/doc/downer.";
    };
    style = lib.mkOption {
      type = lib.types.lines;
      default = "";
      description = "Extra GTK CSS, written to $XDG_CONFIG_HOME/downer/style.css. Catppuccin names (@base, @mauve, @accent…) are in scope.";
    };
    defaultMarkdownViewer = lib.mkEnableOption "Downer as the default handler for Markdown files";
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];
    xdg.configFile."downer/config.ini" = lib.mkIf (cfg.settings != { }) {
      source = ini.generate "downer-config.ini" cfg.settings;
    };
    xdg.configFile."downer/style.css" = lib.mkIf (cfg.style != "") { text = cfg.style; };
    xdg.mimeApps.defaultApplications = lib.mkIf cfg.defaultMarkdownViewer {
      "text/markdown" = [ desktop ];
      "text/x-markdown" = [ desktop ];
    };
  };
}
