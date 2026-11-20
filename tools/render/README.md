# 3D 몸 렌더링과 동작 시험 (tools/render)

콘텐츠용 '보이는 몸'을 시험하는 도구다. 게임 엔진과 따로 Blender 안에서 돈다. 지어낸 구단(HANGANG, 등번호 17)의 오른손 선수 한 명을 MakeHuman 몸으로 만든다. 그 몸으로 정지 자세를 렌더링하거나, CMU 모션 캡처를 입혀 투구·스윙·번트를 움직인다. 실제 선수는 쓰지 않는다.

| 파일 | 하는 일 |
|---|---|
| `compare.py` | 바탕: MakeHuman 몸과 리그(game_engine), 자세 함수, 재질, 구장(마운드·홈플레이트), 빛, 카메라. 단독으로는 간략 인형과 MakeHuman 몸을 같은 자세로 비교 렌더링한다 |
| `refine.py` | 다듬은 몸: 유니폼(파이핑·단추·등번호·가슴 글씨), 언더셔츠, 머리, 모자, 손에 맞춘 글러브, 실밥 있는 공과 포심 그립 |
| `decals.py` | 등번호·가슴 글씨·모자 글자 PNG (Pillow) |
| `cmu_fk.py` | CMU ASF/AMC를 관절 위치로 바꾼다(정운동학). 출력은 Blender 축(Z 위), 미터 |
| `motion.py` | 투구: 관절 위치로 매 프레임 몸을 세운다. 공은 손 속도가 가장 큰 프레임까지 손에 있다가 홈으로 날아간다 |
| `swing.py` | 스윙·번트: 타석에 세우고, 두 주먹을 지나는 직선에 배트를 놓는다 |
| `kin_check.py` | 원본 모션을 120 fps로 잰다(손·배트 속도, 착지에서 놓기까지, 스트라이드, 골반·어깨 회전) |

## 준비 (Linux, 한 번)

```sh
R=$HOME/r3d; mkdir -p $R/dl && cd $R/dl
curl -LO https://download.blender.org/release/Blender4.5/blender-4.5.16-linux-x64.tar.xz
tar xf blender-4.5.16-linux-x64.tar.xz -C $R && mv $R/blender-4.5.16-linux-x64 $R/blender
export BLENDER_USER_RESOURCES=$R/bl_user          # 이 도구만 쓰는 Blender 프로필
# MPFB 2.0.18 (MakeHuman plugin for Blender): https://extensions.blender.org/add-ons/mpfb/ 에서 zip을 받는다
$R/blender/blender -b -c extension install-file -r user_default -e add-on-mpfb-v2.0.18.zip
curl -LO https://files.makehumancommunity.org/asset_packs/makehuman_system_assets/makehuman_system_assets_cc0.zip
unzip -q makehuman_system_assets_cc0.zip -d $BLENDER_USER_RESOURCES/extensions/.user/user_default/mpfb/data
mkdir -p $R/cmu && cd $R/cmu   # CMU subject 124 (120 fps)
for f in 124.asf 124_01.amc 124_02.amc 124_07.amc 124_08.amc; do curl -O http://mocap.cs.cmu.edu/subjects/124/$f; done
python3 tools/render/decals.py $R/decals         # 저장소 루트에서
```

## 쓰는 법

```sh
B="$R/blender/blender -b --python"; T=tools/render
# 정지 자세 (ffc: 앞발 착지, mer: 어깨 최대 외회전)
DECALS=$R/decals $B $T/refine.py -- out/still ffc 64 side,close
# 투구 (30 fps, 36–150 프레임)
python3 -I $T/cmu_fk.py $R/cmu/124.asf $R/cmu/124_01.amc $R/pitch.json 4
MOTION=$R/pitch.json CLIP=36:150 DECALS=$R/decals $B $T/motion.py -- out/pitch ffc 24 side
# 스윙 (60 fps, 짧은 셔터)
python3 -I $T/cmu_fk.py $R/cmu/124.asf $R/cmu/124_07.amc $R/swing60.json 2
MOTION=$R/swing60.json CLIP=80:250 FPS=60 SHUTTER=0.25 DECALS=$R/decals $B $T/swing.py -- out/swing ffc 16 bat_side
# 번트 (세로 근접)
python3 -I $T/cmu_fk.py $R/cmu/124.asf $R/cmu/124_08.amc $R/bunt.json 4
MOTION=$R/bunt.json CLIP=28:100 DECALS=$R/decals $B $T/swing.py -- out/bunt ffc 16 bat_close
# 원본 모션 재기
python3 -I $T/kin_check.py throw $R/cmu/124.asf $R/cmu/124_01.amc
python3 -I $T/kin_check.py swing $R/cmu/124.asf $R/cmu/124_07.amc
```

프레임은 `OUT/카메라/fNNNN.png`로 나온다. 영상은 `ffmpeg -framerate 30 -i out/pitch/side/f%04d.png -c:v libx264 -pix_fmt yuv420p -crf 18 pitch.mp4`로 묶는다. CPU 4코어에서 1280×720, 24샘플 한 프레임이 약 11초 걸린다.

## 몸에 입히는 방식

- 골반: 엉덩이 두 관절과 허리로 만든 좌표계를 쉴 때의 좌표계와 비교해 돌린다. 척추 세 마디는 각 마디 방향을 따르고, 비틀림은 어깨선 방향으로 1/3, 1/2, 전부씩 나눠 준다.
- 팔·다리: 두 마디 IK다. 목표는 손목·발목이고, 팔꿈치·무릎이 굽는 방향을 측정값에서 가져온다. 팔 목표는 팔 길이 비율로 늘린다. 손은 엄지 방향으로, 머리는 얼굴 방향으로 비튼다.
- 크기: CMU 피험자의 다리 길이를 몸(1.90 m)의 다리 길이에 맞춘다(배율 1.141).
- 땅: 매 프레임 낮은 쪽 발이 땅에 닿도록 골반 높이를 고친다. 미끄러짐(수평)은 고치지 않는다.
- 뼈의 행렬을 여러 번 덮어쓰면 반올림이 크기와 위치에 쌓인다. 그래서 회전만 저장해 되돌릴 때 크기와 위치도 쉴 때 값으로 되돌린다. 처음에는 엄지 끝 뼈가 0.73 m까지 늘어나 피부가 찢어졌다.

## 동작 시험 결과 (2026-11-20)

몸에 입히는 과정은 작동한다. 투구 115프레임과 스윙·번트에서 피부가 찢어지지 않고, 발이 땅에 붙으며, 옷·모자·글러브가 따라간다. 하지만 **원본 모션이 선수의 동작이 아니다.** CMU 124번은 투수가 아니라 일반 피험자로 보이고, 측정값이 그렇다.

| 항목 | CMU 124 | 비교 기준 |
|---|---|---|
| 투구 손 속도(공을 놓을 때 공 속도와 거의 같다) | 16.2 m/s (124_01), 14.9 m/s (124_02) | 대학 투수 공 속도 35.3 ± 1.8 m/s, 청소년 30.7 ± 2.7 m/s (Kageyama 2014, 2015) |
| 앞발 착지 때 골반과 어깨선이 목표 쪽으로 돈 정도 | 골반 54°, 어깨 41° (분리 13°) | 대학 투수: 골반 15–16° 열림, 상체 29–34° 닫힘, 차이 약 44–50° (Kageyama 2014 표 3; 2015 표 3) |
| 스트라이드(키 대비) | 79% (124_01), 69% (124_02) | 대학 투수 85.0 ± 3.7%, 청소년 82.9 ± 6.1% (Kageyama 2014, 2015) |
| 다리 들기 | 발끝이 1.15 m까지, 무릎이 엉덩이보다 0.2 m 위로 다리를 뻗어 차올림 | 수치 기준 없음. 무릎을 접어 드는 보통의 투수 동작과 다르다 |
| 스윙 스위트 스폿 속도 | 약 15 m/s (33–34 mph), 두 번의 스윙 | 2025 Statcast 인플레이 타구의 배트 속도 71.2 mph (`docs/physics-model.md`) |

- 각도는 이 도구가 CMU 골격(엉덩이 관절, 쇄골 끝)으로 잰 수평면 각도다. 논문은 마커로 정한 몸통 분절 각도라 정의가 조금 다르다. 소수점이 아니라 크기만 비교한다.
- 손과 배트 속도가 선수의 절반 정도다. 앞발이 닿을 때 상체가 이미 열려 있다(아마추어에게 흔한 '일찍 열림'). 그래서 야구를 아는 사람에게는 '공을 던지는 사람'으로 보이지, '투수'로 보이지 않는다.
- 렌더링 쪽 한계: 공은 38 m/s로 날아가는데 손은 몸 크기에 맞춘 뒤에도 약 18.5 m/s로 움직인다. 그립과 주먹은 고정이다. 배트는 두 주먹을 지나는 직선에 놓으므로 손가락이 배트에서 2–4 cm 떠 있거나 파묻힌다. 글러브는 덩어리에 가깝다. 옷은 몸을 따라 움직이기만 한다(천 시뮬레이션 없음). 디딤발이 조금 미끄러진다.
- 배트 방향은 추정이다. CMU 자료에는 배트 마커가 없다(실제 배트를 들었는지도 적혀 있지 않다). 124_07의 임팩트 직전 4프레임(60 fps, 90–93)에서는 주먹으로 정한 배트가 거의 수직으로 아래를 향한다. 원본 두 손 끝을 잇는 선과는 최대 90° 어긋난다. 경기 재생에서는 배트 경로를 엔진의 스윙(배트 속도, 어택 앵글, 스윙 평면)에서 가져와야 한다.
- 사람 몸을 사실적으로 그릴수록 움직임의 차이를 더 잘 알아챈다(Hodgins 등 1998: 같은 동작 변화를 막대 인형보다 다각형 몸에서 더 잘 구별했다). 몸을 다듬을수록 모션 원본의 질이 더 드러난다.

### 선수다운 모션을 얻는 길

| 길 | 장점 | 걸림돌 |
|---|---|---|
| Driveline OpenBiomechanics (투수 약 100명, 360 Hz 마커) | 실제 투수의 고속 동작 | CC BY-NC-SA 4.0, 영리 목적 금지(수익을 노리는 개인사업자 포함). 수익 채널에 쓰려면 서면 상업 라이선스가 필요하다. 마커를 골격으로 바꾸는 작업도 따로 든다 |
| 상업 모션 마켓의 투구·스윙 클립 | 영상 사용을 허락하는 라이선스가 흔하다 | 클립마다 질을 직접 확인해야 한다 |
| 직접 촬영(실제로 잘 던지는 사람) | 권리 문제가 없다 | 마커 없는 촬영은 빠른 팔 동작에서 정확도가 떨어질 수 있다(검증 필요) |
| 물리 기반 모방 학습(DeepMimic류) | 몸과 공의 물리가 맞는다 | 참조 동작을 따라 하는 방법이라 원본보다 두 배 빠른 동작을 만드는 것은 연구 문제다. GPU가 필요하다 |

## 자료와 권리

- CMU Graphics Lab Motion Capture Database (mocap.cs.cmu.edu): "free for use in research projects. You may include this data in commercially-sold products, but you may not resell this data directly, even in converted form." 출처 표기 문구: "The data used in this project was obtained from mocap.cs.cmu.edu." 저장소에는 넣지 않는다(실행할 때 받는다). Subject 124: 124_01·02 Baseball Pitch, 124_07 Baseball Swing, 124_08 Baseball Bunt.
- MakeHuman 시스템 에셋: CC0 (makehumancommunity.org). MPFB: GPL. Blender: GPL. 이 저장소에는 에셋을 넣지 않는다.

## 출처

- M. Kageyama, T. Sugiyama, Y. Takai, H. Kanehisa, A. Maeda (2014). Kinematic and kinetic profiles of trunk and lower limbs during baseball pitching in collegiate pitchers. *Journal of Sports Science and Medicine* 13(4), 742–750. 표 1–3.
- M. Kageyama, T. Sugiyama, H. Kanehisa, A. Maeda (2015). Difference between adolescent and collegiate baseball pitchers in the kinematics and kinetics of the lower limbs and trunk during pitching motion. *Journal of Sports Science and Medicine* 14, 246–255. 표 1–3.
- J. K. Hodgins, J. F. O'Brien, J. Tumblin (1998). Perception of human motion with different geometric models. *IEEE Transactions on Visualization and Computer Graphics* 4(4), 307–316.
- K. Wasserberger 외 (2022). The OpenBiomechanics Project. Driveline Baseball (github.com/drivelineresearch/openbiomechanics, 라이선스 CC BY-NC-SA 4.0).
- 2025 Statcast 배트 트래킹 표본: [`docs/physics-model.md`](../../docs/physics-model.md) 5.1절과 7절.
