#!/usr/bin/env bash
# This is literally what I used to record this morning.
#
#
# I love FFMPEG, what incredible and ridiculous and always different joys and pains
#
#
#   SCREEN=1 CAMERA=0 MIC=0 ./record.sh session.mkv      # stop with q, which finalises the file
#
# why recordig stopped at 2:00:04:
#   MIC was the macOS default input, which was Bluetooth earbuds on the hands-free profile; at ~2:00 their battery
#   died. macOS logged "Disconnecting SCO", "HFP stopped" and then "Unpublishing audio device …-tsco": the mic device
#   no longer existed.  the screen and the mic share ONE avfoundation input below (`-i "$SCREEN:$MIC"`), so that input
#   stopped delivering anything, video included. Bah!  I could have avoided this, but I guess 3am is not the best time
#   to exercise forethought :-)
set -euo pipefail

SCREEN="${SCREEN:-1}" CAMERA="${CAMERA:-0}" MIC="${MIC:-0}" OUT="${1:-livecode-$(date -u +%Y%m%dT%H%M%SZ).mkv}"

exec ffmpeg -hide_banner \
  -thread_queue_size 1024 -f avfoundation -capture_cursor 1 -framerate 30 -pixel_format nv12 -i "$SCREEN:$MIC" \
  -thread_queue_size 1024 -f avfoundation -framerate 30 -video_size 1280x720 -pixel_format nv12 -i "$CAMERA:none" \
  -filter_complex "[0:v]setpts=PTS-STARTPTS,scale=1920:-2:flags=bilinear[s];[1:v]setpts=PTS-STARTPTS,scale=480:-2:flags=bilinear[c];[s][c]overlay=x=W-w-24:y=H-h-24:eof_action=pass[v];[0:a]asetpts=PTS-STARTPTS[a]" \
  -map "[v]" -map "[a]" -c:a aac -b:a 160k -ar 48000 -ac 2 \
  -fps_mode cfr -r 30 -c:v h264_videotoolbox -realtime 1 -profile:v high \
  -force_key_frames "expr:gte(t,n_forced*2)" -g 60 \
  -b:v 6M -maxrate 6M -bufsize 6M -f matroska "$OUT"


