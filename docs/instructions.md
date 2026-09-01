# git submodule 사용법

1. third_party 폴더로 이동한다.

```bash
cd ./third_party
```

2. git submodule을 이용해 가져온다.

```bash
git submodule add https://github.com/<가져올레포>.git <설치경로/저장할폴더명>
```

3. 가져온 폴더를 확인한다.

```bash
ls <저장할폴더명>
git submodule status
```

# Docker 사용법

## 수동으로 도커 생성

### 도커 생성 명령어

1. ubuntu 이미지 받기

```bash
docker pull ubuntu:24.04
```

확인: `docker images`

2. 컨테이너 생성 후 계속 켜두기

컨테이너 생성

```bash
docker run ubuntu:24.04
```

백그라운드 실행: `-d`
컨테이너 이름 지정: `--name uds-fuzz`
컨테이너 안꺼지게 유지: `sleep infinity`

```bash
docker run -d \
    --name uds-fuzz \
    ubuntu:24.04 \
    sleep infinity
```


3. 실행중인지 확인

```bash
docker ps
```

4. ubuntu 안으로 접속

```bash
docker exec -it uds-fuzz bash
```

5. 기초 사용법

나중에 나가기: `exit`
다시 들어가기: `docker exec -it uds-fuzz bash`
컨테이너 중지: `docker stop uds-fuzz`
컨테이너 시작: `docker start uds-fuzz`
컨테이너 확인: `docker ps`
꺼진 컨테이너 함께 확인: `docker ps -a`
컨테이너 삭제: `docker rm uds-fuzz`

### 한 줄 접속 명령어

interactive, 표준 입력 계속 열어서 명령 입력할 수 있게 함: `-i`
pseudo-TTY, 터미널처럼 보이게 해서 쉘 편하게 이용 가능: `-t`
컨테이너 이름 지정: `--name uds-fuzz`
컨테이너 안꺼지게 유지: `sleep infinity`

```bash
docker run -it \
    --name uds-fuzz \
    ubuntu:24.04 \
    bash
```

삭제: `docker rm uds-fuzz`

## Docker-Compose로 도커 생성

1. compose.yaml 파일을 생성한다.

2. yml 파일에 내용을 삽입한다.

```yaml
services:
  uds-fuzz:
    image: ubuntu:24.04
    container_name: uds-fuzz
    stdin_open: true
    tty: true
    command: bash
```

`services`: 여러 컨테이너 설정을 묶는 최상위 항목
`uds-fuzz`: 사용자가 붙인 서비스 이름, compose 안의 논리적 이름
`image`: ubuntu:24.04 사용 (배포판 선택)
`container_name`: 컨테이너 이름 지정, Docker가 실제로 만들 컨테이너 이름
`stdin_open: true`: docker run에서 `-i`
`tty: true`: docker run에서 `-t`
`command: bash`: 시작 할 프로그램

3. 실행한다.

```bash
docker compose run uds-fuzz
```

### Host 프로젝트 폴더와 Docker를 연결하는 `volume`

```bash
ru
```

`volumes: - .`:
- `/work`: 호스트의 현재 프로젝트 폴더를 Docker 컨테이너 안의 /work 경로에 연결하는 설정
- 그래서 호스트의 해당 프로젝트 파일들을 컨테이너에서 /work 아래에서 그대로 볼 수 있다.
- 컨테이너에서 /work 안의 파일을 수정하면 호스트 파일도 같이 바뀐다.

1. 자동으로 docker compose 파일 찾아 실행

```bash
docker compose up
```

2. 백그라운드에서 실행

```bash
docker compose up -d
```

3. docker 종료

```bash
docker compose down
```

4. bash 접속

`docker compose exec uds-fuzz bash`: 이미 실행 중인 컨테이너 접속
`docker compose run uds-fuzz bash`: 컨테이너 실행하면서 바로 Bash 접속
