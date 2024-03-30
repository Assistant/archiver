{ pkgs
}: {
  format = {
    type = "app";
    program = "${pkgs.format}/bin/format";
  };
}
