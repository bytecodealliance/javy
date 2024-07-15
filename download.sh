#!/bin/bash
TMP_DIR="/tmp/blsjavy"
function cleanup {
	rm -rf $TMP_DIR > /dev/null
}
function fail {
	cleanup
	msg=$1
	echo "============"
	echo "Error: $msg" 1>&2
	exit 1
}
function install {
	#settings
	USER="blocklessnetwork"
	PROG='blsjavy'
	MOVE="true"
	RELEASE="latest"
	INSECURE="false"
	OUT_DIR="$HOME/.bin"
	GH="https://github.com"
	#bash check
	[ ! "$BASH_VERSION" ] && fail "Please use bash instead"
	[ ! -d $OUT_DIR ] && mkdir -p $OUT_DIR
	#dependency check, assume we are a standard POSIX machine
	which find > /dev/null || fail "find not installed"
	which xargs > /dev/null || fail "xargs not installed"
	which sort > /dev/null || fail "sort not installed"
	which tail > /dev/null || fail "tail not installed"
	which cut > /dev/null || fail "cut not installed"
	which du > /dev/null || fail "du not installed"
	GET=""
	if which curl > /dev/null; then
		GET="curl"
		if [[ $INSECURE = "true" ]]; then GET="$GET --insecure"; fi
		GET="$GET --fail -# -L"
	elif which wget > /dev/null; then
		GET="wget"
		if [[ $INSECURE = "true" ]]; then GET="$GET --no-check-certificate"; fi
		GET="$GET -qO-"
	else
		fail "neither wget/curl are installed"
	fi
	#find OS #TODO BSDs and other posixs
	case `uname -s` in
	Darwin) OS="darwin";;
	Linux) OS="linux";;
	*) fail "unknown os: $(uname -s)";;
	esac
	#find ARCH
	if uname -m | grep 64 | grep arm > /dev/null; then
		ARCH="arm64"
	elif uname -m | grep 64 | grep aarch > /dev/null; then
		ARCH="arm64"
	elif uname -m | grep 64 > /dev/null; then
		ARCH="amd64"
	elif uname -m | grep arm > /dev/null; then
		ARCH="arm" #TODO armv6/v7
	elif uname -m | grep 386 > /dev/null; then
		ARCH="386"
	else
		fail "unknown arch: $(uname -m)"
	fi

	#choose from asset list
	URL=""
	FTYPE=""
	DEFAULT_VERSION="v1.4.0"
	VERSION=${1:-$DEFAULT_VERSION} 
	case "${OS}_${ARCH}" in
	"darwin_amd64")
		URL="https://github.com/blocklessnetwork/bls-javy/releases/download/${VERSION}/javy-x86_64-macos-${VERSION}.gz"
		FTYPE=".gz"
		;;
	"darwin_arm64")
		URL="https://github.com/blocklessnetwork/bls-javy/releases/download/${VERSION}/javy-arm-macos-${VERSION}.gz"
		FTYPE=".gz"
		;;
	"linux_amd64")
		URL="https://github.com/blocklessnetwork/bls-javy/releases/download/${VERSION}/javy-x86_64-linux-${VERSION}.gz"
		FTYPE=".gz"
		;;
	"linux_arm64")
		URL="https://github.com/blocklessnetwork/bls-javy/releases/download/${VERSION}/javy-arm-linux-${VERSION}.gz"
		FTYPE=".gz"
		;;
	*) fail "No asset for platform ${OS}-${ARCH}";;
	esac
	#echo the URL
	echo "Download URL: $URL"
	#got URL! download it...
	echo -n "Installing $PROG $VERSION"
	
	echo "....."
	
	#enter tempdir
	mkdir -p $TMP_DIR
	cd $TMP_DIR
	if [[ $FTYPE = ".gz" ]]; then
		which gzip > /dev/null || fail "gzip is not installed"
		#gzipped binary
		NAME="${PROG}_${OS}_${ARCH}.gz"
		#gz download!
		bash -c "$GET $URL" | gzip -d - > $PROG || fail "download failed"
	elif [[ $FTYPE = "" ]]; then
		bash -c "$GET $URL" > "b7s_${OS}_${ARCH}" || fail "download failed"
	else
		fail "unknown file type: $FTYPE"
	fi
	#search subtree largest file (bin)
	TMP_BIN=$(find . -type f | xargs du | sort -n | tail -n 1 | cut -f 2)
	if [ ! -f "$TMP_BIN" ]; then
		fail "could not find binary (largest file)"
	fi
	#ensure its larger than 1MB
	if [[ $(du -m $TMP_BIN | cut -f1) -lt 1 ]]; then
		fail "no binary found ($TMP_BIN is not larger than 1MB)"
	fi
	#move into PATH or cwd
	chmod +x $TMP_BIN || fail "chmod +x failed"
	
	mv $TMP_BIN $OUT_DIR/$PROG || fail "mv failed" #FINAL STEP!
	echo "Installed $PROG $VERSION at $OUT_DIR/$PROG"
	#done
	cleanup
	echo "Please add the following line to your .bashrc or .zshrc:"
	echo 'export PATH="$HOME/.bin:$PATH"'
}
install $1