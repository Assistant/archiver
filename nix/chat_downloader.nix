{ lib
, fetchFromGitHub
, python3Packages
}:
python3Packages.buildPythonApplication rec {
  pname = "chat_downloader";
  version = "0.2.8";

  src = fetchFromGitHub {
    owner = "xenova";
    repo = "chat-downloader";
    rev = "v${version}";
    sha256 = "sha256-wnfS45avwYm+DdTMiYpxJrsV06PWdOO+Bssq3FaNJow=";
  };

  buildInputs = [ python3Packages.setuptools ];
  propagatedBuildInputs = with python3Packages;
    [ requests colorlog docstring-parser isodate websocket-client ];
  doCheck = false;

  meta = with lib; {
    homepage = "https://github.com/xenova/chat-downloader";
    description = "A simple tool used to retrieve chat messages from livestreams, videos, clips and past broadcasts. No authentication needed!";
    license = licenses.mit;
  };
}
