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
    defaultMarkdownViewer = lib.mkEnableOption "Downer as the default handler for Markdown files";
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    xdg.mime.defaultApplications = lib.mkIf cfg.defaultMarkdownViewer {
      "text/markdown" = desktop;
      "text/x-markdown" = desktop;
    };
  };
}
