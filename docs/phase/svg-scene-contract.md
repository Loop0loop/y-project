# SVG 씬 계약

Project-Y에서 SVG는 Rust 코드가 디자인하는 대상이 아니라, 외부 그래픽 씬 파일이다. Rust는 상태값을 주입하고 래스터화해서 Kitty에 올리는 역할만 맡는다.

## 책임 분리

- Gemini/디자인 쪽: `assets/svg/*.svg` 작성, 형태/색/타이포/장식 결정
- Rust 로직 쪽: 게임 상태 계산, 토큰 주입, 터미널 픽셀 크기 산출, `resvg` 래스터화, Kitty 전송
- 공통 계약: SVG는 `{{TOKEN}}` 플레이스홀더만 사용한다

## SVG 루트 규칙

```xml
<svg width="{{WIDTH}}" height="{{HEIGHT}}" viewBox="0 0 1200 675" xmlns="http://www.w3.org/2000/svg">
```

- `viewBox`는 16:9 기준 `1200x675`로 고정한다.
- 실제 출력 크기는 Rust가 `{{WIDTH}}`, `{{HEIGHT}}`로 주입한다.
- 반응형은 SVG 내부 좌표를 다시 계산하지 않고, `viewBox` 스케일링으로 처리한다.

## 허용 토큰

- `{{WIDTH}}`, `{{HEIGHT}}`: 실제 래스터 크기
- `{{PROGRESS}}`: `0..100` 정수
- `{{PROGRESS_BAR_WIDTH}}`: 500px 기준 진행 바 폭
- `{{PHASE_LABEL}}`, `{{TITLE}}`, `{{SUBTITLE}}`, `{{BODY}}`: 화면 텍스트

## SVG 작성 제한

- 사용 가능: `path`, `circle`, `polygon`, `rect`, `text`, `pattern`, `linearGradient`, `clipPath`, 기본 `filter`
- 피하기: `foreignObject`, 외부 이미지 URL, CSS 애니메이션, JS, 웹폰트 다운로드
- 이유: `resvg` 래스터화와 터미널 출력에서 재현성이 중요하다

## 현재 우선순위

1. Splash 씬 드롭인 검증
2. Training 씬 토큰 확장
3. Court/Dating 씬 분리
4. 씬 전환 애니메이션은 마지막
