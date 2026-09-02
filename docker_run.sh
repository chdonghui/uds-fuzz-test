#!/usr/bin/env bash

# 명령 실패 시 즉시 스크립트 종료
set -e

# 이 스크립트가 있는 프로젝트 폴더로 이동
cd "$(dirname "$0")"

# 컨테이너가 없으면 생성, 꺼져 있으면 실행
docker compose up -d

# uds-fuzz 컨테이너에 bash로 접속
docker compose exec uds-fuzz bash
